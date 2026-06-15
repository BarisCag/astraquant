use astra_core::events::{EventType, PayloadEncoding, PayloadMetadata};
use astra_core::exchange::ExchangeRuntime;
use astra_core::hashing::DeterministicState;
use astra_core::journal::EventJournal;
use astra_core::kernel::AstraKernel;
use astra_core::orderbook::{LimitOrderPlacedPayload, OrderSide};
use astra_core::replay::{EventReducer, ReplayEngine};
use astra_core::risk::create_default_risk_engine;
use astra_core::runtime::StrategyRuntime;
use astra_core::serialization::serialize_canonical;
use astra_core::types::{Money, Price, Quantity};
use std::fs;
use std::path::PathBuf;

fn temp_path(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_artifacts");
    fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn cleanup(path: &std::path::Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn test_exchange_limit_order_and_replay_equivalence() {
    let jl_path = temp_path("exchange_wiring.astra_jl");
    cleanup(&jl_path);

    let limits =
        create_default_risk_engine(Money::new(10_000_000_000_000), Quantity::new(1_000_000_000));

    let mut runtime = ExchangeRuntime::new(limits.clone());
    runtime.add_market("BTC/USD".to_string());

    let mut journal = EventJournal::create(&jl_path, 1_700_000_000_000_000_000).unwrap();

    let ask = LimitOrderPlacedPayload {
        order_id: 1,
        trader_id: 1,
        symbol: "BTC/USD".to_string(),
        side: OrderSide::Ask,
        price: Price::new(500_000_000),
        quantity: Quantity::new(1_0000),
    };
    let event1 = journal
        .commit(
            1_700_000_000_000_000_000,
            EventType::LimitOrderPlaced,
            serialize_canonical(&ask).unwrap(),
            PayloadMetadata::new(PayloadEncoding::Bincode, 1),
        )
        .unwrap();
    runtime.apply(&event1).unwrap();

    let bid = LimitOrderPlacedPayload {
        order_id: 2,
        trader_id: 1,
        symbol: "BTC/USD".to_string(),
        side: OrderSide::Bid,
        price: Price::new(500_000_000),
        quantity: Quantity::new(1_0000),
    };
    let event2 = journal
        .commit(
            1_700_000_000_000_000_001,
            EventType::LimitOrderPlaced,
            serialize_canonical(&bid).unwrap(),
            PayloadMetadata::new(PayloadEncoding::Bincode, 1),
        )
        .unwrap();
    runtime.apply(&event2).unwrap();

    assert_eq!(runtime.ledger.trades.len(), 1);
    let original_hash = runtime.state_hash();

    let mut recovered = ExchangeRuntime::new(limits);
    recovered.add_market("BTC/USD".to_string());

    ReplayEngine::replay_and_verify(
        &EventJournal::open(&jl_path).unwrap(),
        &mut recovered,
        original_hash,
    )
    .unwrap();

    assert_eq!(recovered.ledger.trades.len(), 1);
    assert_eq!(recovered.state_hash(), original_hash);
    cleanup(&jl_path);
}

#[test]
fn test_exchange_risk_rejects_oversized_order() {
    let limits = create_default_risk_engine(Money::new(1_000), Quantity::new(10));
    let mut runtime = ExchangeRuntime::new(limits);
    runtime.add_market("BTC/USD".to_string());

    let payload = LimitOrderPlacedPayload {
        order_id: 99,
        trader_id: 1,
        symbol: "BTC/USD".to_string(),
        side: OrderSide::Bid,
        price: Price::new(100),
        quantity: Quantity::new(100),
    };

    let event = astra_core::events::AstraEvent::new(
        1,
        1,
        EventType::LimitOrderPlaced,
        serialize_canonical(&payload).unwrap(),
        PayloadMetadata::new(PayloadEncoding::Bincode, 1),
    );

    assert!(runtime.apply(&event).is_err());
    assert_eq!(runtime.ledger.trades.len(), 0);
}

#[test]
fn test_kernel_chain_matching_portfolio_ledger_replay() {
    let jl_path = temp_path("kernel_chain.astra_jl");
    cleanup(&jl_path);

    let limits =
        create_default_risk_engine(Money::new(10_000_000_000_000), Quantity::new(1_000_000_000));

    let mut exchange = ExchangeRuntime::new(limits.clone());
    exchange.add_market("ETH/USD".to_string());
    let mut kernel = AstraKernel::new(StrategyRuntime::new(exchange));

    let mut journal = EventJournal::create(&jl_path, 1_700_000_000_000_000_000).unwrap();

    let sell = LimitOrderPlacedPayload {
        order_id: 10,
        trader_id: 1,
        symbol: "ETH/USD".to_string(),
        side: OrderSide::Ask,
        price: Price::new(30_000_000),
        quantity: Quantity::new(5000),
    };
    kernel
        .apply(
            &journal
                .commit(
                    1_700_000_000_000_000_000,
                    EventType::LimitOrderPlaced,
                    serialize_canonical(&sell).unwrap(),
                    PayloadMetadata::new(PayloadEncoding::Bincode, 1),
                )
                .unwrap(),
        )
        .unwrap();

    let buy = LimitOrderPlacedPayload {
        order_id: 11,
        trader_id: 1,
        symbol: "ETH/USD".to_string(),
        side: OrderSide::Bid,
        price: Price::new(30_000_000),
        quantity: Quantity::new(5000),
    };
    kernel
        .apply(
            &journal
                .commit(
                    1_700_000_000_000_000_001,
                    EventType::LimitOrderPlaced,
                    serialize_canonical(&buy).unwrap(),
                    PayloadMetadata::new(PayloadEncoding::Bincode, 1),
                )
                .unwrap(),
        )
        .unwrap();

    assert_eq!(kernel.strategy_runtime.exchange.ledger.trades.len(), 1);
    let hash_live = kernel.state_hash();

    let mut recovered = AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits)));
    recovered
        .strategy_runtime
        .exchange
        .add_market("ETH/USD".to_string());

    let result = ReplayEngine::replay_and_verify(
        &EventJournal::open(&jl_path).unwrap(),
        &mut recovered,
        hash_live,
    )
    .unwrap();

    assert_eq!(result.events_applied, 2);
    assert_eq!(recovered.strategy_runtime.exchange.ledger.trades.len(), 1);
    assert_eq!(recovered.state_hash(), hash_live);
    cleanup(&jl_path);
}
