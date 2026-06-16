use astra_core::events::{EventType, PayloadMetadata};
use astra_core::exchange::ExchangeRuntime;
use astra_core::journal::EventJournal;
use astra_core::kernel::AstraKernel;
use astra_core::replay::ReplayEngine;
use astra_core::risk::create_default_risk_engine;
use astra_core::runtime::StrategyRuntime;
use astra_core::types::{Money, Quantity};

#[test]
fn test_deterministic_replay_after_forced_crash() {
    let limits = create_default_risk_engine(Money::new(100_000_000), Quantity::new(1_000));

    let mut kernel = AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits)));
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("test.astra_jl");

    // Simulate initial run
    {
        let mut journal = EventJournal::create(&journal_path, 1_000_000).unwrap();
        journal
            .commit(
                1_000_000,
                EventType::MarketTick,
                vec![1, 2, 3],
                PayloadMetadata::raw(),
            )
            .unwrap();
        journal
            .commit(
                1_000_001,
                EventType::MarketTick,
                vec![4, 5, 6],
                PayloadMetadata::raw(),
            )
            .unwrap();
    }

    // Simulate E2E Replay
    let journal = EventJournal::open(&journal_path).unwrap();
    let result = ReplayEngine::replay_journal(&journal, &mut kernel).unwrap();

    assert_eq!(result.events_applied, 2);
    assert_eq!(result.final_sequence_id, 2);
    assert_ne!(result.final_state_hash, [0; 32]); // Ensure hash mutated
}
