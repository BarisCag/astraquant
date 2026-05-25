use astra_core::clock::VirtualClock;
use astra_core::dataset::Dataset;
use astra_core::exchange::ExchangeRuntime;
use astra_core::feed::HistoricalFeed;
use astra_core::hashing::DeterministicState;
use astra_core::journal::EventJournal;
use astra_core::kernel::AstraKernel;
use astra_core::marketdata::MarketTick;
use astra_core::replay::ReplayEngine;
use astra_core::risk::RiskLimits;
use astra_core::runtime::StrategyRuntime;
use astra_core::simulation::SimulationRuntime;
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
fn test_simulation_deterministic_replay_equivalence() {
    let dataset_path = temp_path("sim_btcusd.astra_ds");
    let jl_path = temp_path("simulation.astra_jl");
    cleanup(&dataset_path);
    cleanup(&jl_path);

    let ticks = vec![
        MarketTick {
            symbol: "BTC/USD".to_string(),
            timestamp_ns: 1_700_000_000_000_000_000,
            bid_price: Price::new(500_000_000),
            ask_price: Price::new(500_010_000),
            bid_quantity: Quantity::new(10000),
            ask_quantity: Quantity::new(10000),
        },
        MarketTick {
            symbol: "BTC/USD".to_string(),
            timestamp_ns: 1_700_000_001_000_000_000,
            bid_price: Price::new(500_020_000),
            ask_price: Price::new(500_030_000),
            bid_quantity: Quantity::new(20000),
            ask_quantity: Quantity::new(20000),
        },
    ];

    Dataset::save(dataset_path.to_str().unwrap(), &ticks).unwrap();
    let feed = HistoricalFeed::new(Dataset::load(dataset_path.to_str().unwrap()).unwrap());
    let clock = VirtualClock::new(1_700_000_000_000_000_000);

    let limits = RiskLimits::new(Money::new(10_000_000_000_000), Quantity::new(1_000_000_000));

    let kernel = AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits.clone())));
    let journal = EventJournal::create(&jl_path, 1_700_000_000_000_000_000).unwrap();

    let mut sim = SimulationRuntime::new(kernel, feed, clock, journal);
    sim.run_all().unwrap();

    let live_hash = sim.kernel.state_hash();
    assert_eq!(sim.journal.len(), 2);

    let mut recovered = AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits)));
    ReplayEngine::replay_and_verify(
        &EventJournal::open(&jl_path).unwrap(),
        &mut recovered,
        live_hash,
    )
    .unwrap();

    assert_eq!(recovered.state_hash(), live_hash);
    cleanup(&dataset_path);
    cleanup(&jl_path);
}
