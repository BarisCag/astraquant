use crate::events::AstraEvent;
use crate::hashing::{hash_bytes, DeterministicState};
use crate::serialization::serialize_canonical;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateTransitionProof {
    pub pre_state_hash: [u8; 32],
    pub event_hash: [u8; 32],
    pub post_state_hash: [u8; 32],
}

impl StateTransitionProof {
    pub fn generate(pre: [u8; 32], event: &AstraEvent, post: [u8; 32]) -> Self {
        Self {
            pre_state_hash: pre,
            event_hash: hash_bytes(&serialize_canonical(event).unwrap()),
            post_state_hash: post,
        }
    }

    pub fn verify(&self, expected_post: &[u8; 32]) -> bool {
        self.post_state_hash == *expected_post
            && self.event_hash != [0u8; 32]
            && self.pre_state_hash != self.post_state_hash
    }
}

impl DeterministicState for StateTransitionProof {
    fn state_hash(&self) -> [u8; 32] {
        hash_bytes(&serialize_canonical(self).unwrap())
    }
}
