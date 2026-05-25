use astra_core::events::EventType;
use astra_core::exchange::ExchangeRuntime;
use astra_core::gateway::ExecutionGateway;
use astra_core::hashing::DeterministicState;
use astra_core::journal::EventJournal;
use astra_core::kernel::AstraKernel;
use astra_core::replay::ReplayEngine;
use astra_core::risk::RiskLimits;
use astra_core::runtime::StrategyRuntime;
use astra_core::types::{Money, Quantity};
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
fn test_gateway_journals_and_replays_kernel_state() {
    let jl_path = temp_path("gateway.astra_jl");
    cleanup(&jl_path);

    let limits = RiskLimits::new(Money::new(10_000_000_000_000), Quantity::new(1_000_000_000));

    let kernel = AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits.clone())));
    let journal = EventJournal::create(&jl_path, 1_700_000_000_000_000_000).unwrap();
    let mut gateway = ExecutionGateway::new(kernel, journal);

    gateway
        .ingest_raw_event(
            1_700_000_000_000_000_000,
            EventType::MarketTick,
            vec![7, 8, 9],
        )
        .unwrap();

    let initial_hash = gateway.kernel.state_hash();
    assert_eq!(gateway.journal.len(), 1);

    let mut recovered = AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits)));
    ReplayEngine::replay_and_verify(
        &EventJournal::open(&jl_path).unwrap(),
        &mut recovered,
        initial_hash,
    )
    .unwrap();

    assert_eq!(recovered.state_hash(), initial_hash);
    cleanup(&jl_path);
}
