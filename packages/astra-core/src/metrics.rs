use crate::hashing::{hash_bytes, DeterministicState};
use crate::serialization::serialize_canonical;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct KernelMetrics {
    pub total_events_processed: u64,
    pub active_actors: u64,
    pub error_count: u64,
}

impl Default for KernelMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl KernelMetrics {
    pub fn new() -> Self {
        Self {
            total_events_processed: 0,
            active_actors: 0,
            error_count: 0,
        }
    }
}

impl DeterministicState for KernelMetrics {
    fn state_hash(&self) -> [u8; 32] {
        hash_bytes(&serialize_canonical(self).unwrap())
    }
}
