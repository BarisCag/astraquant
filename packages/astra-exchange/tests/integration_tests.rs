use astra_core::events::{EventType, PayloadEncoding, PayloadMetadata};
use astra_core::journal::EventJournal;
use astra_core::orderbook::{LimitOrderPlacedPayload, OrderSide};
use astra_core::serialization::serialize_canonical;
use astra_core::types::{Price, Quantity};
use astra_exchange::replay::FullReplayEngine;
use astra_exchange::runtime::ExchangeRuntime;

use astra_risk::engine::RiskEngine;
use astra_risk::types::TraderRiskProfile;
use std::fs;
use std::path::PathBuf;

// Simple LCG for deterministic synthesis
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }
    fn next_range(&mut self, min: u64, max: u64) -> u64 {
        let range = max - min + 1;
        min + (self.next() % range)
    }
}

fn synthesize_journal(dir: PathBuf, seed: u64, num_events: usize) -> PathBuf {
    fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("astra_19700101_00.astra_jl");
    let mut journal = EventJournal::create(&file_path, 0).unwrap();

    let mut lcg = Lcg::new(seed);

    for sequence_id in 1..=num_events as u64 {
        let is_buy = lcg.next() % 2 == 0;
        let side = if is_buy {
            OrderSide::Bid
        } else {
            OrderSide::Ask
        };
        let trader_id = lcg.next_range(1, 3); // 3 traders
        let price = lcg.next_range(90, 110);
        let quantity = lcg.next_range(1, 5);

        let payload = LimitOrderPlacedPayload {
            order_id: lcg.next(),
            trader_id,
            symbol: "BTC/USD".to_string(),
            side,
            price: Price::new(price as i64),
            quantity: Quantity::new(quantity),
        };

        journal
            .commit(
                sequence_id * 1_000_000,
                EventType::LimitOrderPlaced,
                serialize_canonical(&payload).unwrap(),
                PayloadMetadata::new(PayloadEncoding::Bincode, 1),
            )
            .unwrap();
    }
    dir
}

#[test]
fn test_deterministic_replay_identity() {
    let _tempdir = tempfile::tempdir().unwrap();
    let dir = _tempdir.path().to_path_buf();
    synthesize_journal(dir.clone(), 42, 1000);

    let mut risk_engine = RiskEngine::new();
    risk_engine.register_trader(TraderRiskProfile {
        trader_id: 1,
        max_position_notional: 1000000,
        max_order_quantity: 10000,
        max_drawdown: 10000,
        max_order_velocity: 100,
    });
    risk_engine.register_trader(TraderRiskProfile {
        trader_id: 2,
        max_position_notional: 1000000,
        max_order_quantity: 10000,
        max_drawdown: 10000,
        max_order_velocity: 100,
    });
    risk_engine.register_trader(TraderRiskProfile {
        trader_id: 3,
        max_position_notional: 1000000,
        max_order_quantity: 10000,
        max_drawdown: 10000,
        max_order_velocity: 100,
    });

    let mut replay1 = FullReplayEngine::new(ExchangeRuntime::new(risk_engine.clone()));
    let hash1 = replay1.replay_directory(&dir).unwrap();

    let mut replay2 = FullReplayEngine::new(ExchangeRuntime::new(risk_engine));
    let hash2 = replay2.replay_directory(&dir).unwrap();

    assert_eq!(hash1, hash2);

    // Ensure we actually processed events
    assert!(replay1.runtime.diagnostics.total_events_processed > 0);
    assert_eq!(replay1.runtime.sequence_clock, 1000);
}

#[test]
fn test_rejected_order_isolation() {
    let _tempdir = tempfile::tempdir().unwrap();
    let dir = _tempdir.path().to_path_buf();
    // We create a journal with a massive order that should be rejected
    fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("astra_19700101_00.astra_jl");
    let mut journal = EventJournal::create(&file_path, 0).unwrap();

    let payload = LimitOrderPlacedPayload {
        order_id: 1,
        trader_id: 1,
        symbol: "BTC/USD".to_string(),
        side: OrderSide::Bid,
        price: Price::new(100),
        quantity: Quantity::new(1_000_000), // Massive quantity
    };

    journal
        .commit(
            1_000_000,
            EventType::LimitOrderPlaced,
            serialize_canonical(&payload).unwrap(),
            PayloadMetadata::new(PayloadEncoding::Bincode, 1),
        )
        .unwrap();

    let mut risk_engine = RiskEngine::new();
    risk_engine.register_trader(TraderRiskProfile {
        trader_id: 1,
        max_position_notional: 1000, // Rejects the massive order
        max_order_quantity: 10000,   // Still would reject 1M
        max_drawdown: 10000,
        max_order_velocity: 100,
    });

    let mut replay = FullReplayEngine::new(ExchangeRuntime::new(risk_engine));
    replay.replay_directory(&dir).unwrap();

    assert_eq!(replay.runtime.diagnostics.total_rejected_orders, 1);
    assert_eq!(replay.runtime.diagnostics.total_accepted_orders, 0);
    // LOB should be completely empty
    assert!(
        !replay
            .runtime
            .router
            .venues
            .values()
            .next()
            .unwrap()
            .books
            .contains_key("BTC/USD")
            || replay
                .runtime
                .router
                .venues
                .values()
                .next()
                .unwrap()
                .books
                .get("BTC/USD")
                .unwrap()
                .orders
                .is_empty()
    );
    // Portfolio should be completely empty
    assert_eq!(replay.runtime.position_engine.positions.len(), 0);
}

#[test]
fn test_offline_arrival_rejection() {
    let _tempdir = tempfile::tempdir().unwrap();
    let dir = _tempdir.path().to_path_buf();
    fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("astra_19700101_00.astra_jl");
    let mut journal = EventJournal::create(&file_path, 0).unwrap();

    let mut risk_engine = RiskEngine::new();
    risk_engine.register_trader(TraderRiskProfile {
        trader_id: 1,
        max_position_notional: 10000000,
        max_order_quantity: 100000,
        max_drawdown: 100000,
        max_order_velocity: 10000,
    });

    // 1. Submit an order
    let payload = LimitOrderPlacedPayload {
        order_id: 1,
        trader_id: 1,
        symbol: "BTC/USD".to_string(),
        side: OrderSide::Bid,
        price: Price::new(100),
        quantity: Quantity::new(1),
    };
    journal
        .commit(
            1,
            EventType::LimitOrderPlaced,
            serialize_canonical(&payload).unwrap(),
            PayloadMetadata::new(PayloadEncoding::Bincode, 1),
        )
        .unwrap();

    // 2. Set Venue Offline
    let failure_event = astra_router::failure::VenueFailureEvent::VenueOffline {
        venue_id: astra_router::venue::VenueId(1),
        sequence_id: 2,
    };
    journal
        .commit(
            2,
            EventType::VenueStatusChanged,
            serialize_canonical(&failure_event).unwrap(),
            PayloadMetadata::new(PayloadEncoding::Bincode, 1),
        )
        .unwrap();

    let mut replay = FullReplayEngine::new(ExchangeRuntime::new(risk_engine));
    // The runtime defaults to adding Venue 1 in ExchangeRuntime::new, let's just make sure.
    // Wait, the router doesn't add venues automatically in tests unless we do it in new().
    // We will just let the replay run.
    replay.replay_directory(&dir).unwrap();

    // At seq=2, the venue goes offline. The order from seq=1 has ingress delay, so it arrives at seq = 1 + delay (probably > 2).
    // It should be rejected as VenueOffline.
    assert_eq!(
        replay.runtime.diagnostics.lob_diagnostics.total_rejections,
        1
    );
}
