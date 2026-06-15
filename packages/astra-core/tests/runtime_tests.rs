use astra_core::events::{EventType, PayloadEncoding, PayloadMetadata};
use astra_core::exchange::ExchangeRuntime;
use astra_core::hashing::DeterministicState;
use astra_core::journal::EventJournal;
use astra_core::marketdata::MarketTick;
use astra_core::replay::{EventReducer, ReplayEngine};
use astra_core::risk::create_default_risk_engine;
use astra_core::runtime::{AnyStrategy, StrategyRuntime};
use astra_core::strategies::mean_reversion::MeanReversionStrategy;
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
fn test_strategy_runtime_state_mutates_and_replays() {
    let jl_path = temp_path("runtime.astra_jl");
    cleanup(&jl_path);

    let limits =
        create_default_risk_engine(Money::new(10_000_000_000_000), Quantity::new(1_000_000_000));

    let mut runtime = StrategyRuntime::new(ExchangeRuntime::new(limits.clone()));
    runtime.add_strategy(AnyStrategy::MeanReversion(MeanReversionStrategy::new(
        1,
        "BTC/USD".to_string(),
        10,
        50,
    )));

    let mut journal = EventJournal::create(&jl_path, 1_700_000_000_000_000_000).unwrap();
    let tick = MarketTick {
        symbol: "BTC/USD".to_string(),
        timestamp_ns: 1_700_000_000_000_000_000,
        bid_price: Price::new(100),
        ask_price: Price::new(101),
        bid_quantity: Quantity::new(1),
        ask_quantity: Quantity::new(1),
    };

    runtime
        .apply(
            &journal
                .commit(
                    1_700_000_000_000_000_000,
                    EventType::MarketTick,
                    astra_core::serialization::serialize_canonical(&tick).unwrap(),
                    PayloadMetadata::new(PayloadEncoding::Bincode, 1),
                )
                .unwrap(),
        )
        .unwrap();

    let AnyStrategy::MeanReversion(strategy) = runtime.strategies.get(&1).unwrap();
    assert_eq!(strategy.tick_count, 1);
    let original_hash = runtime.state_hash();

    let mut recovered = StrategyRuntime::new(ExchangeRuntime::new(limits));
    recovered.add_strategy(AnyStrategy::MeanReversion(MeanReversionStrategy::new(
        1,
        "BTC/USD".to_string(),
        10,
        50,
    )));

    ReplayEngine::replay_and_verify(
        &EventJournal::open(&jl_path).unwrap(),
        &mut recovered,
        original_hash,
    )
    .unwrap();

    assert_eq!(
        match recovered.strategies.get(&1).unwrap() {
            AnyStrategy::MeanReversion(s) => s.tick_count,
        },
        1
    );
    cleanup(&jl_path);
}
