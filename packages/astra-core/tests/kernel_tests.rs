use astra_core::events::{EventType, PayloadEncoding, PayloadMetadata};
use astra_core::exchange::ExchangeRuntime;
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
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_artifacts");
    fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn cleanup(path: &std::path::Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn test_kernel_recovery_deterministic() {
    let jl_path = temp_path("ph5_kernel.astra_jl");
    cleanup(&jl_path);

    let limits =
        create_default_risk_engine(Money::new(10_000_000_000_000), Quantity::new(1_000_000_000));

    let runtime = StrategyRuntime::new(ExchangeRuntime::new(limits.clone()));
    let mut kernel = AstraKernel::new(runtime);

    let mut journal = EventJournal::create(&jl_path, 1_700_000_000_000_000_000).unwrap();

    let event = journal
        .commit(
            1_700_000_000_000_000_000,
            EventType::MarketTick,
            vec![1, 2, 3],
            PayloadMetadata::new(PayloadEncoding::RawBytes, 1),
        )
        .unwrap();

    kernel.apply(&event).unwrap();

    let original_hash = kernel.state_hash();

    // Recover
    let mut recovered_kernel = AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits)));

    let journal_read = EventJournal::open(&jl_path).unwrap();
    let result = ReplayEngine::replay_and_verify_from(
        &journal_read,
        &mut recovered_kernel,
        0,
        original_hash,
    )
    .unwrap();

    assert_eq!(result.verified, Some(true));

    cleanup(&jl_path);
}
