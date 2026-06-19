use astra_core::events::{AstraEvent, EventType};
use astra_core::gateway::ExecutionGateway;
use astra_core::journal::EventJournal;
use astra_core::marketdata::MarketTick;
use astra_core::types::{Price, Quantity};
use astra_core::hashing::{DeterministicState, hash_to_hex};
use std::fs;

use astra_paper::engine::PaperEngine;
use astra_paper::execution::{PaperExecutionEngine, FillModel};
use astra_paper::portfolio::PortfolioTracker;
use astra_paper::risk::{RiskEngine, RiskLimits};
use astra_paper::strategy::TwapStrategy;
use astra_paper::types::Side;

fn generate_synthetic_crash_data() -> Vec<AstraEvent> {
    let mut events = Vec::new();
    let mut price: i64 = 50_000_000_000; // 500 USD
    let mut timestamp_ns = 1_000_000_000_000;
    
    // 1 hour = 3600 seconds
    for i in 0..3600 {
        if i > 1800 && i <= 1860 {
            // Flash crash: drop 20% in 60s -> ~ 166,666,666 per second
            price -= 166_666_666;
        } else if i > 1860 && i <= 2400 {
            // Partial recovery
            price += 15_000_000;
        } else {
            // Random walk drift would break determinism unless seeded, just keep it flat or tiny drift
            if i % 2 == 0 {
                price += 1_000_000;
            } else {
                price -= 1_000_000;
            }
        }

        let tick = MarketTick {
            symbol: "BTCUSDT".to_string(),
            timestamp_ns,
            bid_price: Price((price - 100) as i64),
            ask_price: Price((price + 100) as i64),
            bid_quantity: Quantity(1_000_000),
            ask_quantity: Quantity(1_000_000),
        };

        let payload = astra_core::serialization::serialize_canonical(&tick).unwrap();

        events.push(AstraEvent::new_raw(
            timestamp_ns,
            i as u64,
            EventType::MarketTick,
            payload,
        ));

        timestamp_ns += 1_000_000_000; // 1 sec
    }

    events
}

fn run_engine(journal_path: &str, events: &[AstraEvent]) -> [u8; 32] {
    let mut journal = EventJournal::create(journal_path, 0).unwrap();
    let mut gateway = ExecutionGateway::new(journal);
    
    let portfolio = PortfolioTracker::new(100_000_000_000_000);
    let execution_engine = PaperExecutionEngine::new(FillModel::Slippage { n_trades: 5, slippage_bps: 2 });
    let risk_limits = RiskLimits {
        max_notional_per_symbol: 50_000_000_000_000,
        max_drawdown_usd: 5_000_000_000_000,
        max_orders_per_second: 10,
    };
    let risk_engine = RiskEngine::new(risk_limits, 100_000_000_000_000);
    
    let twap = Box::new(TwapStrategy::new(
        "BTCUSDT".to_string(),
        Side::Buy,
        1_000_000_000,     
        100_000_000,       
        60_000_000_000,    
        0,                 
    ));

    let mut engine = PaperEngine::new(portfolio, execution_engine, risk_engine, twap as Box<dyn astra_paper::strategy::Strategy>);

    let mut sequence_counter = gateway.journal.next_sequence_id();
    let mut last_hash = [0; 32];

    for event in events {
        let mut e = event.clone();
        e.sequence_id = sequence_counter;
        gateway.journal.append(&e).unwrap();
        last_hash = e.state_hash();
        sequence_counter += 1;

        let out_events = engine.process_event(&e, &last_hash);
        for mut out_event in out_events {
            out_event.sequence_id = sequence_counter;
            gateway.journal.append(&out_event).unwrap();
            last_hash = out_event.state_hash();
            sequence_counter += 1;
        }
    }
    
    last_hash
}

#[test]
fn test_deterministic_replay() {
    let events = generate_synthetic_crash_data();

    let run1_path = "run1.astra_jl";
    let run2_path = "run2.astra_jl";

    let _ = fs::remove_file(run1_path);
    let _ = fs::remove_file(run2_path);

    let hash1 = run_engine(run1_path, &events);
    let hash2 = run_engine(run2_path, &events);

    assert_eq!(hash1, hash2, "Replay determinism failed! Hashes differ.");
    
    let _ = fs::remove_file(run1_path);
    let _ = fs::remove_file(run2_path);
}
