use crate::events::AstraEvent;
use crate::hashing::{hash_bytes, DeterministicState};
use crate::serialization::serialize_canonical;

pub struct ReplicationBuffer {
    pub pending_events: Vec<AstraEvent>,
}

impl Default for ReplicationBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplicationBuffer {
    pub fn new() -> Self {
        Self {
            pending_events: Vec::new(),
        }
    }

    pub fn push(&mut self, event: AstraEvent) {
        self.pending_events.push(event);
    }
}

impl DeterministicState for ReplicationBuffer {
    fn state_hash(&self) -> [u8; 32] {
        hash_bytes(&serialize_canonical(&self.pending_events).unwrap())
    }
}
