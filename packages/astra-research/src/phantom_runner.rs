//! Phantom Kernel Runner
//!
//! Loads a .astra_ds dataset, replays it through an isolated AstraKernel,
//! and records state_hash() at every 100 events.
//! Outputs hash_trace.json.

use astra_core::exchange::ExchangeRuntime;
use astra_core::hashing::{hash_to_hex, DeterministicState};
use astra_core::kernel::AstraKernel;
use astra_core::marketdata::MarketTick;
use astra_core::replay::EventReducer;
use astra_core::risk::create_default_risk_engine;
use astra_core::runtime::StrategyRuntime;
use astra_core::serialization::deserialize_canonical;
use astra_core::types::{Money, Quantity};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::dataset_format::{CrisisDataset, DatasetReader};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashTraceEntry {
    pub sequence_id: u64,
    pub state_hash: String,
    pub price_raw: i64,
    pub volume_raw: u64,
}

use astra_core::merkle::MerkleTree;

pub struct PhantomRunner {
    pub kernel: AstraKernel,
}

impl PhantomRunner {
    pub fn new() -> Self {
        let limits = create_default_risk_engine(Money::new(1_000_000_000), Quantity::new(100_000));
        let kernel = AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits)));
        Self { kernel }
    }

    /// Replay all events from a CrisisDataset. Returns hash trace entries
    /// and Merkle roots captured every `checkpoint_interval` events.
    pub fn run(
        &mut self,
        dataset: &CrisisDataset,
        checkpoint_interval: u64,
    ) -> (Vec<HashTraceEntry>, Vec<String>) {
        let mut trace: Vec<HashTraceEntry> = Vec::new();
        let mut merkle_roots: Vec<String> = Vec::new();
        let mut events_processed = 0u64;
        let mut current_hashes = Vec::with_capacity(checkpoint_interval as usize);

        for event in &dataset.events {
            let _ = self.kernel.apply(event);
            events_processed += 1;
            current_hashes.push(self.kernel.state_hash());

            if events_processed % checkpoint_interval == 0 {
                let tree = MerkleTree::build(&current_hashes);
                if let Some(root) = tree.root_hash() {
                    merkle_roots.push(hash_to_hex(&root));
                }
                current_hashes.clear();

                // Extract price from payload if it's a MarketTick
                let (price_raw, volume_raw) =
                    if let Ok(tick) = deserialize_canonical::<MarketTick>(&event.payload) {
                        (tick.bid_price.0, tick.bid_quantity.0)
                    } else {
                        (0, 0)
                    };

                trace.push(HashTraceEntry {
                    sequence_id: event.sequence_id,
                    state_hash: hash_to_hex(&self.kernel.state_hash()),
                    price_raw,
                    volume_raw,
                });
            }
        }

        // Always capture final state
        let last = dataset.events.last();
        if let Some(evt) = last {
            let (price_raw, volume_raw) =
                if let Ok(tick) = deserialize_canonical::<MarketTick>(&evt.payload) {
                    (tick.bid_price.0, tick.bid_quantity.0)
                } else {
                    (0, 0)
                };
            trace.push(HashTraceEntry {
                sequence_id: evt.sequence_id,
                state_hash: hash_to_hex(&self.kernel.state_hash()),
                price_raw,
                volume_raw,
            });

            if !current_hashes.is_empty() {
                let tree = MerkleTree::build(&current_hashes);
                if let Some(root) = tree.root_hash() {
                    merkle_roots.push(hash_to_hex(&root));
                }
            }
        }

        (trace, merkle_roots)
    }

    pub fn final_hash(&self) -> [u8; 32] {
        self.kernel.state_hash()
    }
}
