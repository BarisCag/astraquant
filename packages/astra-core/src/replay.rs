//! Deterministic replay and cryptographic verification subsystem.
//!
//! The `ReplayEngine` is responsible for restoring the exact application state from
//! the `EventJournal`. It applies events sequentially to an `EventReducer` and recalculates
//! the expected cryptographic `state_hash`. If the hash deviates from the known
//! deterministic root, the engine fails closed to prevent silent data corruption.

use crate::events::AstraEvent;
use crate::hashing::DeterministicState;

pub trait EventReducer: crate::hashing::DeterministicState {
    type Error: std::fmt::Display;
    fn apply(&mut self, event: &AstraEvent) -> Result<(), Self::Error>;
    fn last_applied_sequence_id(&self) -> Option<u64>;
}

pub struct ReplayResult {
    pub final_sequence_id: u64,
    pub final_state_hash: [u8; 32],
    pub verified: Option<bool>,
    pub events_applied: u64,
}

pub struct ReplayEngine<'a, R: EventReducer + DeterministicState> {
    reducer: &'a mut R,
}

impl<'a, R: EventReducer + DeterministicState> ReplayEngine<'a, R> {
    pub fn new(reducer: &'a mut R) -> Self {
        Self { reducer }
    }

    // =========================================================================
    // Static convenience methods (match test API expectations)
    // =========================================================================

    /// Replay all events from a journal into a reducer. Static convenience method.
    pub fn replay_journal(
        journal: &crate::journal::EventJournal,
        reducer: &'a mut R,
    ) -> Result<ReplayResult, std::io::Error> {
        let mut engine = Self::new(reducer);
        engine.replay_iter(journal.iter()?)
    }

    /// Replay and verify against an expected hash. Static convenience method.
    pub fn replay_and_verify(
        journal: &crate::journal::EventJournal,
        reducer: &'a mut R,
        expected_hash: [u8; 32],
    ) -> Result<ReplayResult, std::io::Error> {
        let mut engine = Self::new(reducer);
        let mut res = engine.replay_iter(journal.iter()?)?;
        let verified = res.final_state_hash == expected_hash;
        res.verified = Some(verified);
        if !verified {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Replay verification failed: expected {:?}, got {:?}",
                    expected_hash, res.final_state_hash
                ),
            ));
        }
        Ok(res)
    }

    /// Replay from a checkpoint sequence_id (events after start_seq). Static convenience method.
    pub fn replay_from(
        journal: &crate::journal::EventJournal,
        reducer: &'a mut R,
        start_seq: u64,
    ) -> Result<ReplayResult, std::io::Error> {
        let mut engine = Self::new(reducer);
        engine.replay_iter(journal.iter_from(start_seq)?)
    }

    /// Replay and verify from a checkpoint. Static convenience method.
    pub fn replay_and_verify_from(
        journal: &crate::journal::EventJournal,
        reducer: &'a mut R,
        start_seq: u64,
        expected_hash: [u8; 32],
    ) -> Result<ReplayResult, std::io::Error> {
        let mut engine = Self::new(reducer);
        let mut res = engine.replay_iter(journal.iter_from(start_seq)?)?;
        let verified = res.final_state_hash == expected_hash;
        res.verified = Some(verified);
        if !verified {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Replay verification failed: expected {:?}, got {:?}",
                    expected_hash, res.final_state_hash
                ),
            ));
        }
        Ok(res)
    }

    // =========================================================================
    // Instance method (generic over any iterator)
    // =========================================================================

    /// Replay events from any iterator of AstraEvent results.
    pub fn replay_iter<I: Iterator<Item = Result<AstraEvent, std::io::Error>>>(
        &mut self,
        events: I,
    ) -> Result<ReplayResult, std::io::Error> {
        let mut last_seq = 0;
        let mut events_applied = 0;
        for event_res in events {
            let event = event_res?;
            self.reducer
                .apply(&event)
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            last_seq = event.sequence_id;
            events_applied += 1;
        }

        Ok(ReplayResult {
            final_sequence_id: last_seq,
            final_state_hash: self.reducer.state_hash(),
            verified: None,
            events_applied,
        })
    }
}
