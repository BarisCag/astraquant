use astra_core::events::{EventType, PayloadEncoding, PayloadMetadata};
use astra_core::exchange::ExchangeRuntime;
use astra_core::hashing::DeterministicState;
use astra_core::journal::EventJournal;
use astra_core::kernel::AstraKernel;
use astra_core::orchestrator::VmOrchestrator;
use astra_core::package::StrategyPackage;
use astra_core::replay::{EventReducer, ReplayEngine};
use astra_core::risk::create_default_risk_engine;
use astra_core::runtime::StrategyRuntime;
use astra_core::types::{Money, Quantity};
use astra_core::vm::DeterministicVm;
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
fn test_vm_recovery_deterministic() {
    let jl_path = temp_path("ph9_vm.astra_jl");
    cleanup(&jl_path);

    let limits =
        create_default_risk_engine(Money::new(10_000_000_000_000), Quantity::new(1_000_000_000));

    let runtime = StrategyRuntime::new(ExchangeRuntime::new(limits.clone()));
    let kernel = AstraKernel::new(runtime);
    let mut orchestrator = VmOrchestrator::new(kernel);

    let package = StrategyPackage::new("AlphaStrategy".to_string(), vec![0x00, 0x61, 0x73, 0x6D]);
    let vm = DeterministicVm::load(package.clone(), 1_000_000).unwrap();
    orchestrator.register_vm(1, vm);

    let mut journal = EventJournal::create(&jl_path, 1_700_000_000_000_000_000).unwrap();

    let event = journal
        .commit(
            1_700_000_000_000_000_000,
            EventType::MarketTick,
            vec![1, 2, 3],
            PayloadMetadata::new(PayloadEncoding::RawBytes, 1),
        )
        .unwrap();

    orchestrator.apply(&event).unwrap();

    let initial_hash = orchestrator.state_hash();
    let initial_gas = orchestrator.vms.get(&1).unwrap().sandbox.gas_meter.used_gas;

    // Recover
    let mut recovered_orch = VmOrchestrator::new(AstraKernel::new(StrategyRuntime::new(
        ExchangeRuntime::new(limits),
    )));
    let recovered_vm = DeterministicVm::load(package, 1_000_000).unwrap();
    recovered_orch.register_vm(1, recovered_vm);

    let journal_read = EventJournal::open(&jl_path).unwrap();

    let result =
        ReplayEngine::replay_and_verify_from(&journal_read, &mut recovered_orch, 0, initial_hash)
            .unwrap();

    assert_eq!(result.verified, Some(true));
    assert_eq!(
        recovered_orch
            .vms
            .get(&1)
            .unwrap()
            .sandbox
            .gas_meter
            .used_gas,
        initial_gas
    );

    assert_eq!(recovered_orch.state_hash(), initial_hash);
    cleanup(&jl_path);
}
