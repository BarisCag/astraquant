use astra_core::events::EventType;
use astra_core::exchange::ExchangeRuntime;
use astra_core::gateway::ExecutionGateway;
use astra_core::hashing::DeterministicState;
use astra_core::journal::EventJournal;
use astra_core::kernel::AstraKernel;
use astra_core::replay::{EventReducer, ReplayEngine};
use astra_core::risk::create_default_risk_engine;
use astra_core::runtime::StrategyRuntime;
use astra_core::types::{Money, Quantity};
use std::fs;
use std::path::PathBuf;

fn temp_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "astra_core_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn cleanup(path: &std::path::Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn test_gateway_journals_and_replays_kernel_state() {
    let jl_path = temp_path("gateway.astra_jl");
    cleanup(&jl_path);

    let limits =
        create_default_risk_engine(Money::new(10_000_000_000_000), Quantity::new(1_000_000_000));

    let mut kernel = AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits.clone())));
    let journal = EventJournal::create(&jl_path, 1_700_000_000_000_000_000).unwrap();
    let mut gateway = ExecutionGateway::new(journal);

    gateway
        .ingest_raw_event(
            1_700_000_000_000_000_000,
            EventType::MarketTick,
            vec![7, 8, 9],
        )
        .unwrap();

    let event = gateway.next_event().unwrap();
    kernel.apply(&event).unwrap();

    let initial_hash = kernel.state_hash();
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
