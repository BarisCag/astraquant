use crate::clock::VirtualClock;
use crate::events::{EventType, PayloadEncoding, PayloadMetadata};
use crate::feed::HistoricalFeed;
use crate::hashing::{hash_bytes, DeterministicState};
use crate::journal::EventJournal;
use crate::kernel::AstraKernel;
use crate::replay::EventReducer;
use crate::serialization::serialize_canonical;

pub struct SimulationRuntime {
    pub kernel: AstraKernel,
    pub feed: HistoricalFeed,
    pub clock: VirtualClock,
    pub journal: EventJournal,
}

impl SimulationRuntime {
    pub fn new(
        kernel: AstraKernel,
        feed: HistoricalFeed,
        clock: VirtualClock,
        journal: EventJournal,
    ) -> Self {
        Self {
            kernel,
            feed,
            clock,
            journal,
        }
    }

    pub fn step(&mut self) -> Result<bool, String> {
        if let Some(tick) = self.feed.next_tick() {
            self.clock.advance_to(tick.timestamp_ns);

            let payload = serialize_canonical(tick).map_err(|e| e.to_string())?;

            let event = self
                .journal
                .commit(
                    self.clock.current_time_ns,
                    EventType::MarketTick,
                    payload,
                    PayloadMetadata::new(PayloadEncoding::Bincode, 1),
                )
                .map_err(|e| e.to_string())?;

            self.kernel.apply(&event).map_err(|e| e.to_string())?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn run_all(&mut self) -> Result<(), String> {
        while self.step()? {}
        Ok(())
    }
}

impl DeterministicState for SimulationRuntime {
    fn state_hash(&self) -> [u8; 32] {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.kernel.state_hash());
        bytes.extend_from_slice(&self.feed.state_hash());
        bytes.extend_from_slice(&self.clock.state_hash());
        hash_bytes(&bytes)
    }
}
