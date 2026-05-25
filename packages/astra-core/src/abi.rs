use crate::events::AstraEvent;
use crate::hashing::{hash_bytes, DeterministicState};
use crate::serialization::serialize_canonical;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum HostCall {
    EmitEvent(AstraEvent),
    Log(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VmStateDump {
    pub memory_hash: [u8; 32],
    pub execution_pointer: u64,
}

impl DeterministicState for VmStateDump {
    fn state_hash(&self) -> [u8; 32] {
        hash_bytes(&serialize_canonical(self).unwrap())
    }
}
