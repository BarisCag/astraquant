use astra_core::gateway::ExecutionGateway;
use astra_core::journal::EventJournal;
use std::path::Path;

use crate::binance::BinanceFeed;
use crate::normalizer::TradeNormalizer;
use astra_core::hashing::DeterministicState;

pub struct LiveFeedManager {
    gateway: ExecutionGateway,
    sequence_counter: u64,
    last_hash: [u8; 32],
}

impl LiveFeedManager {
    pub fn new(journal_path: &str) -> Self {
        let journal = if Path::new(journal_path).exists() {
            EventJournal::open(journal_path).unwrap()
        } else {
            EventJournal::create(journal_path, 0).unwrap()
        };
        let sequence_counter = journal.next_sequence_id();
        let gateway = ExecutionGateway::new(journal);
        Self {
            gateway,
            sequence_counter,
            last_hash: [0; 32],
        }
    }

    pub async fn run(&mut self, symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
        let feed = BinanceFeed::new(symbol);
        let mut stream = feed.connect().await?;

        while let Some(raw_trade) = stream.next_trade().await {
            let event = TradeNormalizer::normalize(&raw_trade, self.sequence_counter);
            
            // Log to journal (ingest_raw_event assigns its own sequence based on journal, but here we do it to simulate gateway ingest)
            self.gateway.journal.append(&event)?;
            
            self.last_hash = event.state_hash();
            
            println!("[SEQ {:04}] {} {} | {} | hash: 0x{}...", 
                self.sequence_counter, 
                raw_trade.symbol, 
                raw_trade.price_str, 
                raw_trade.quantity_str, 
                &astra_core::hashing::hash_to_hex(&self.last_hash)[..8]
            );

            self.sequence_counter += 1;
        }

        Ok(())
    }

    pub fn events_processed(&self) -> u64 {
        self.sequence_counter.saturating_sub(1)
    }

    pub fn current_state_hash(&self) -> [u8; 32] {
        self.last_hash
    }
}
