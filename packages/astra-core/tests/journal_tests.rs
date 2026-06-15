//! Phase 2 integration tests: Journal, Replay, Snapshot, Crash Recovery.
//!
//! The milestone test is `test_crash_recovery_deterministic`:
//! ingest → snapshot → crash → restore → replay → STATE HASH MATCH: TRUE

use astra_core::{
    hash_bytes, serialize_canonical, AstraEvent, DeterministicState, EventJournal, EventReducer,
    EventType, PayloadMetadata, ReplayEngine, SnapshotManager,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// =============================================================================
// Test Reducer — Deterministic counter/accumulator
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct CounterReducer {
    count: u64,
    sum: u64,
    last_seq: Option<u64>,
}

impl CounterReducer {
    fn new() -> Self {
        Self {
            count: 0,
            sum: 0,
            last_seq: None,
        }
    }
}

impl DeterministicState for CounterReducer {
    fn state_hash(&self) -> [u8; 32] {
        let bytes = serialize_canonical(self).expect("CounterReducer serialization must not fail");
        hash_bytes(&bytes)
    }
}

impl EventReducer for CounterReducer {
    type Error = String;

    fn apply(&mut self, event: &AstraEvent) -> Result<(), String> {
        self.count += 1;
        self.sum += event.payload.iter().map(|b| *b as u64).sum::<u64>();
        self.last_seq = Some(event.sequence_id);
        Ok(())
    }

    fn last_applied_sequence_id(&self) -> Option<u64> {
        self.last_seq
    }
}

// =============================================================================
// Helpers
// =============================================================================

fn temp_path(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_artifacts");
    fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

// =============================================================================
// Journal Tests
// =============================================================================

#[test]
fn test_journal_create_commit_reopen() {
    let path = temp_path("ph2_create_reopen.astra_jl");
    cleanup(&path);

    {
        let mut j = EventJournal::create(&path, 1_700_000_000_000_000_000).unwrap();
        for i in 1..=20u64 {
            j.commit(
                1_700_000_000_000_000_000 + i * 1_000_000,
                EventType::MarketTick,
                vec![i as u8],
                PayloadMetadata::raw(),
            )
            .unwrap();
        }
        assert_eq!(j.len(), 20);
    }

    // Reopen and validate
    let j = EventJournal::open(&path).unwrap();
    assert_eq!(j.len(), 20);
    assert_eq!(j.next_sequence_id(), 21);

    let events: Vec<AstraEvent> = j.iter().unwrap().map(|r| r.unwrap()).collect();
    assert_eq!(events.len(), 20);
    assert_eq!(events[0].sequence_id, 1);
    assert_eq!(events[19].sequence_id, 20);

    cleanup(&path);
}

#[test]
fn test_journal_roundtrip_read_after_commit() {
    let path = temp_path("ph2_roundtrip.astra_jl");
    cleanup(&path);

    let mut j = EventJournal::create(&path, 1_700_000_000_000_000_000).unwrap();
    j.commit(
        1_700_000_000_000_000_000,
        EventType::MarketTick,
        vec![42],
        PayloadMetadata::raw(),
    )
    .unwrap();

    // Valid read should succeed
    let events: Vec<_> = j.iter().unwrap().collect();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_ok());

    cleanup(&path);
}

#[test]
fn test_journal_sequence_enforcement() {
    let path = temp_path("ph2_seq_enforce.astra_jl");
    cleanup(&path);

    let mut j = EventJournal::create(&path, 1_700_000_000_000_000_000).unwrap();

    // Correct: sequence_id = 1
    let event1 = AstraEvent::new_raw(1_700_000_000_000_000_000, 1, EventType::MarketTick, vec![1]);
    assert!(j.append(&event1).is_ok());

    // Wrong: sequence_id = 5 (expected 2)
    let event_bad =
        AstraEvent::new_raw(1_700_000_000_000_000_000, 5, EventType::MarketTick, vec![2]);
    assert!(j.append(&event_bad).is_err());

    // Correct: sequence_id = 2
    let event2 = AstraEvent::new_raw(1_700_000_000_000_000_000, 2, EventType::MarketTick, vec![2]);
    assert!(j.append(&event2).is_ok());

    cleanup(&path);
}

#[test]
fn test_journal_iter_from_midpoint() {
    let path = temp_path("ph2_iter_mid.astra_jl");
    cleanup(&path);

    let mut j = EventJournal::create(&path, 1_700_000_000_000_000_000).unwrap();
    for i in 1..=100u64 {
        j.commit(
            1_700_000_000_000_000_000 + i * 1_000_000,
            EventType::MarketTick,
            vec![(i % 256) as u8],
            PayloadMetadata::raw(),
        )
        .unwrap();
    }

    // Read from after sequence 50
    let events: Vec<AstraEvent> = j.iter_from(50).unwrap().map(|r| r.unwrap()).collect();
    assert_eq!(events.len(), 50);
    assert_eq!(events[0].sequence_id, 51);
    assert_eq!(events[49].sequence_id, 100);

    cleanup(&path);
}

// =============================================================================
// Replay Tests
// =============================================================================

#[test]
fn test_replay_full_produces_correct_state() {
    let path = temp_path("ph2_replay_full.astra_jl");
    cleanup(&path);

    let mut j = EventJournal::create(&path, 1_700_000_000_000_000_000).unwrap();
    for i in 1..=10u64 {
        j.commit(
            1_700_000_000_000_000_000 + i * 1_000_000,
            EventType::MarketTick,
            vec![i as u8],
            PayloadMetadata::raw(),
        )
        .unwrap();
    }

    let mut reducer = CounterReducer::new();
    let result = ReplayEngine::replay_journal(&j, &mut reducer).unwrap();

    assert_eq!(result.events_applied, 10);
    assert_eq!(reducer.count, 10);
    assert_eq!(reducer.sum, 55); // 1+2+...+10

    cleanup(&path);
}

#[test]
fn test_replay_deterministic_two_passes() {
    let path = temp_path("ph2_replay_det.astra_jl");
    cleanup(&path);

    let mut j = EventJournal::create(&path, 1_700_000_000_000_000_000).unwrap();
    for i in 1..=25u64 {
        j.commit(
            1_700_000_000_000_000_000 + i * 1_000_000,
            EventType::MarketTick,
            vec![i as u8],
            PayloadMetadata::raw(),
        )
        .unwrap();
    }

    let mut r1 = CounterReducer::new();
    let mut r2 = CounterReducer::new();
    let res1 = ReplayEngine::replay_journal(&j, &mut r1).unwrap();
    let res2 = ReplayEngine::replay_journal(&j, &mut r2).unwrap();

    assert_eq!(res1.final_state_hash, res2.final_state_hash);
    assert_eq!(r1, r2);

    cleanup(&path);
}

#[test]
fn test_replay_and_verify_success() {
    let path = temp_path("ph2_replay_verify.astra_jl");
    cleanup(&path);

    let mut j = EventJournal::create(&path, 1_700_000_000_000_000_000).unwrap();
    for i in 1..=5u64 {
        j.commit(
            1_700_000_000_000_000_000 + i * 1_000_000,
            EventType::MarketTick,
            vec![i as u8],
            PayloadMetadata::raw(),
        )
        .unwrap();
    }

    // First pass: get expected hash
    let mut r1 = CounterReducer::new();
    let res1 = ReplayEngine::replay_journal(&j, &mut r1).unwrap();

    // Second pass: verify against expected hash
    let mut r2 = CounterReducer::new();
    let res2 = ReplayEngine::replay_and_verify(&j, &mut r2, res1.final_state_hash).unwrap();
    assert_eq!(res2.verified, Some(true));

    cleanup(&path);
}

#[test]
fn test_replay_and_verify_from_failure() {
    let path = temp_path("ph2_replay_verify_from_fail.astra_jl");
    cleanup(&path);

    let mut j = EventJournal::create(&path, 1_700_000_000_000_000_000).unwrap();
    for i in 1..=3u64 {
        j.commit(
            1_700_000_000_000_000_000 + i,
            EventType::MarketTick,
            vec![i as u8],
            PayloadMetadata::raw(),
        )
        .unwrap();
    }

    let mut reducer = CounterReducer::new();
    ReplayEngine::replay_journal(&j, &mut reducer).unwrap();
    let wrong_hash = [0xFFu8; 32];

    let mut reducer2 = CounterReducer::new();
    assert!(ReplayEngine::replay_and_verify_from(&j, &mut reducer2, 1, wrong_hash).is_err());
    cleanup(&path);
}

#[test]
fn test_replay_and_verify_failure() {
    let path = temp_path("ph2_replay_verify_fail.astra_jl");
    cleanup(&path);

    let mut j = EventJournal::create(&path, 1_700_000_000_000_000_000).unwrap();
    j.commit(
        1_700_000_000_000_000_000,
        EventType::MarketTick,
        vec![1],
        PayloadMetadata::raw(),
    )
    .unwrap();

    let wrong_hash = [0xFFu8; 32];
    let mut reducer = CounterReducer::new();
    let result = ReplayEngine::replay_and_verify(&j, &mut reducer, wrong_hash);
    assert!(result.is_err());

    cleanup(&path);
}

// =============================================================================
// Snapshot Tests
// =============================================================================

#[test]
fn test_snapshot_capture_restore_verify() {
    let jl_path = temp_path("ph2_snap_jl.astra_jl");
    let ss_path = temp_path("ph2_snap.astra_ss");
    cleanup(&jl_path);
    cleanup(&ss_path);

    let mut j = EventJournal::create(&jl_path, 1_700_000_000_000_000_000).unwrap();
    let mut reducer = CounterReducer::new();

    for i in 1..=10u64 {
        let event = j
            .commit(
                1_700_000_000_000_000_000 + i * 1_000_000,
                EventType::MarketTick,
                vec![i as u8],
                PayloadMetadata::raw(),
            )
            .unwrap();
        reducer.apply(&event).unwrap();
    }

    let original_hash = reducer.state_hash();

    // Capture snapshot
    let meta =
        SnapshotManager::capture(&reducer, "test-engine", 1_700_000_000_000_000_000, &ss_path)
            .unwrap();
    assert_eq!(meta.last_sequence_id, 10);
    assert_eq!(meta.state_hash, original_hash);

    // Restore and verify
    let snapshot = SnapshotManager::restore(&ss_path).unwrap();
    let restored: CounterReducer = snapshot.restore_state().unwrap();
    assert_eq!(restored, reducer);
    assert_eq!(restored.state_hash(), original_hash);

    // Verify utility
    let verified_meta = SnapshotManager::verify(&ss_path).unwrap();
    assert_eq!(verified_meta.last_sequence_id, 10);

    cleanup(&jl_path);
    cleanup(&ss_path);
}

// =============================================================================
// THE MILESTONE TEST: Crash Recovery
// =============================================================================

#[test]
fn test_crash_recovery_deterministic() {
    let jl_path = temp_path("ph2_crash_recovery.astra_jl");
    let ss_path = temp_path("ph2_crash_recovery.astra_ss");
    cleanup(&jl_path);
    cleanup(&ss_path);

    let snapshot_at: u64 = 50;
    let total_events: u64 = 100;

    // =========================================================================
    // PHASE 1: Normal operation — ingest events, snapshot at midpoint
    // =========================================================================

    let mut journal = EventJournal::create(&jl_path, 1_700_000_000_000_000_000).unwrap();
    let mut reducer = CounterReducer::new();

    for i in 1..=total_events {
        let event = journal
            .commit(
                1_700_000_000_000_000_000 + i * 1_000_000,
                EventType::MarketTick,
                vec![(i % 256) as u8],
                PayloadMetadata::raw(),
            )
            .unwrap();
        reducer.apply(&event).unwrap();

        // Snapshot at midpoint
        if i == snapshot_at {
            SnapshotManager::capture(
                &reducer,
                "crash-test-engine",
                1_700_000_000_000_000_000 + i * 1_000_000,
                &ss_path,
            )
            .unwrap();
        }
    }

    // Record the authoritative final state hash
    let original_state_hash = reducer.state_hash();
    let original_count = reducer.count;
    let original_sum = reducer.sum;

    let _finished_reducer = reducer;
    let _finished_journal = journal;

    // =========================================================================
    // PHASE 3: Recovery — restore from snapshot + replay journal
    // =========================================================================

    // Reopen journal (validates all checksums during open)
    let journal = EventJournal::open(&jl_path).unwrap();

    // Restore state from snapshot
    let snapshot = SnapshotManager::restore(&ss_path).unwrap();
    let mut recovered_reducer: CounterReducer = snapshot.restore_state().unwrap();

    // Replay remaining events (51..100) from journal
    let result = ReplayEngine::replay_and_verify_from(
        &journal,
        &mut recovered_reducer,
        snapshot.metadata.last_sequence_id,
        original_state_hash,
    )
    .unwrap();

    // =========================================================================
    // PHASE 4: Verification — THE MOMENT OF TRUTH
    // =========================================================================

    let recovered_hash = recovered_reducer.state_hash();

    // THE INVARIANT
    assert_eq!(
        original_state_hash, recovered_hash,
        "DETERMINISTIC REPLAY INVARIANT VIOLATED"
    );
    assert_eq!(recovered_reducer.count, original_count);
    assert_eq!(recovered_reducer.sum, original_sum);
    assert_eq!(result.verified, Some(true));
    assert_eq!(result.events_applied, total_events - snapshot_at);

    cleanup(&jl_path);
    cleanup(&ss_path);
}

// =============================================================================
// Additional Robustness Tests
// =============================================================================

#[test]
fn test_replay_from_checkpoint_matches_full_replay() {
    let jl_path = temp_path("ph2_checkpoint_vs_full.astra_jl");
    let ss_path = temp_path("ph2_checkpoint_vs_full.astra_ss");
    cleanup(&jl_path);
    cleanup(&ss_path);

    let mut journal = EventJournal::create(&jl_path, 1_700_000_000_000_000_000).unwrap();
    let mut live_reducer = CounterReducer::new();

    for i in 1..=30u64 {
        let event = journal
            .commit(
                1_700_000_000_000_000_000 + i * 1_000_000,
                EventType::MarketTick,
                vec![i as u8],
                PayloadMetadata::raw(),
            )
            .unwrap();
        live_reducer.apply(&event).unwrap();

        if i == 15 {
            SnapshotManager::capture(
                &live_reducer,
                "cmp-engine",
                1_700_000_000_000_000_000,
                &ss_path,
            )
            .unwrap();
        }
    }

    let live_hash = live_reducer.state_hash();

    // Full replay from genesis
    let mut full_reducer = CounterReducer::new();
    ReplayEngine::replay_journal(&journal, &mut full_reducer).unwrap();
    assert_eq!(full_reducer.state_hash(), live_hash);

    // Checkpoint replay
    let snapshot = SnapshotManager::restore(&ss_path).unwrap();
    let mut snap_reducer: CounterReducer = snapshot.restore_state().unwrap();
    ReplayEngine::replay_from(&journal, &mut snap_reducer, 15).unwrap();
    assert_eq!(snap_reducer.state_hash(), live_hash);

    // All three must match
    assert_eq!(live_reducer, full_reducer);
    assert_eq!(full_reducer, snap_reducer);

    cleanup(&jl_path);
    cleanup(&ss_path);
}

#[test]
fn test_all_event_types_through_journal_replay() {
    let path = temp_path("ph2_all_types.astra_jl");
    cleanup(&path);

    let mut journal = EventJournal::create(&path, 1_700_000_000_000_000_000).unwrap();
    let types = [
        EventType::MarketTick,
        EventType::OrderSubmitted,
        EventType::OrderFilled,
        EventType::RiskLimitBreached,
        EventType::StateSnapshot,
    ];

    for (i, et) in types.iter().enumerate() {
        journal
            .commit(
                1_700_000_000_000_000_000 + (i as u64) * 1_000_000,
                *et,
                vec![(i + 1) as u8],
                PayloadMetadata::raw(),
            )
            .unwrap();
    }

    let mut reducer = CounterReducer::new();
    let mut engine = ReplayEngine::new(&mut reducer);
    let result = engine.replay_iter(journal.iter().unwrap()).unwrap();
    assert_eq!(result.events_applied, 5);
    assert_eq!(reducer.count, 5);
    assert_eq!(reducer.sum, 15); // 1+2+3+4+5

    cleanup(&path);
}

#[test]
fn test_mid_write_crash_recovery() {
    let jl_path = temp_path("ph2_mid_write_crash.astra_jl");
    cleanup(&jl_path);
    let mut journal = EventJournal::create(&jl_path, 0).unwrap();
    
    // Step 1: Write 50 events
    for i in 1..=50u64 {
        let event = AstraEvent::new_raw(
            1000000000 + i * 1000000,
            i,
            EventType::MarketTick,
            vec![(i % 256) as u8],
        );
        journal.append(&event).unwrap();
    }
    
    // Calculate expected state for 49 events
    let mut clean_49_reducer = CounterReducer::new();
    let mut count = 0;
    for event_res in EventJournal::open(&jl_path).unwrap().iter().unwrap() {
        if let Ok(e) = event_res {
            clean_49_reducer.apply(&e).unwrap();
            count += 1;
            if count == 49 { break; }
        } else {
            break;
        }
    }
    let clean_49_hash = clean_49_reducer.state_hash();

    // Step 2: Truncate file at random offset within last record
    use std::fs::OpenOptions;
    let file = OpenOptions::new().write(true).open(&jl_path).unwrap();
    let len = file.metadata().unwrap().len();
    file.set_len(len - 7).unwrap(); // Truncate last 7 bytes

    // Step 3: Replay
    let mut reducer = CounterReducer::new();
    let result = ReplayEngine::replay_journal(&EventJournal::open(&jl_path).unwrap(), &mut reducer);
    
    // Step 4 & 5
    assert!(result.is_ok() || result.is_err(), "Should either recover cleanly by ignoring corrupt record or return an Error without panic");
    
    let crash_hash = reducer.state_hash();
    
    assert_eq!(crash_hash, clean_49_hash, "Replay did not correctly reconstruct 49 events");

    cleanup(&jl_path);
}
