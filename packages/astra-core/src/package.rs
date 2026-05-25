use crate::hashing::{hash_bytes, verify_hash_equality, DeterministicState};
use crate::serialization::serialize_canonical;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StrategyPackage {
    pub name: String,
    pub wasm_bytes: Vec<u8>,
    pub checksum: [u8; 32],
}

impl StrategyPackage {
    pub fn new(name: String, wasm_bytes: Vec<u8>) -> Self {
        let checksum = hash_bytes(&wasm_bytes);
        Self {
            name,
            wasm_bytes,
            checksum,
        }
    }

    pub fn verify(&self) -> bool {
        let actual = hash_bytes(&self.wasm_bytes);
        verify_hash_equality(&self.checksum, &actual)
    }
}

impl DeterministicState for StrategyPackage {
    fn state_hash(&self) -> [u8; 32] {
        hash_bytes(&serialize_canonical(self).unwrap())
    }
}
