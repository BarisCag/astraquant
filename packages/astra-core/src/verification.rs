use crate::hashing::{hash_bytes, DeterministicState};
use crate::serialization::serialize_canonical;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationManifest {
    pub node_id: u64,
    pub sequence_id: u64,
    pub state_hash: [u8; 32],
}

impl DeterministicState for VerificationManifest {
    fn state_hash(&self) -> [u8; 32] {
        hash_bytes(&serialize_canonical(self).unwrap())
    }
}

pub fn verify_cluster_hashes(manifests: &[VerificationManifest]) -> bool {
    if manifests.is_empty() {
        return true;
    }
    let first = &manifests[0].state_hash;
    manifests.iter().all(|m| m.state_hash == *first)
}
