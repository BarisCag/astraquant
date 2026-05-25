use crate::hashing::{hash_bytes, DeterministicState};
use crate::serialization::serialize_canonical;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionArtifact {
    pub session_id: String,
    pub start_time_ns: u64,
    pub end_time_ns: u64,
    pub initial_state_hash: [u8; 32],
    pub final_state_hash: [u8; 32],
}

impl DeterministicState for SessionArtifact {
    fn state_hash(&self) -> [u8; 32] {
        hash_bytes(&serialize_canonical(self).unwrap())
    }
}
