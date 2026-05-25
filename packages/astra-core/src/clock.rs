use crate::hashing::{hash_bytes, DeterministicState};
use crate::serialization::serialize_canonical;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VirtualClock {
    pub current_time_ns: u64,
}

impl VirtualClock {
    pub fn new(start_ns: u64) -> Self {
        Self {
            current_time_ns: start_ns,
        }
    }

    pub fn advance_to(&mut self, time_ns: u64) {
        if time_ns > self.current_time_ns {
            self.current_time_ns = time_ns;
        }
    }
}

impl DeterministicState for VirtualClock {
    fn state_hash(&self) -> [u8; 32] {
        hash_bytes(&serialize_canonical(self).expect("VirtualClock serialization failed"))
    }
}
