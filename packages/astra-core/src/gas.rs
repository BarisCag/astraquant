use crate::hashing::{hash_bytes, DeterministicState};
use crate::serialization::serialize_canonical;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GasMeter {
    pub max_gas: u64,
    pub used_gas: u64,
}

impl GasMeter {
    pub fn new(max_gas: u64) -> Self {
        Self {
            max_gas,
            used_gas: 0,
        }
    }

    pub fn consume(&mut self, amount: u64) -> Result<(), String> {
        if self.used_gas + amount > self.max_gas {
            return Err("Out of gas".to_string());
        }
        self.used_gas += amount;
        Ok(())
    }
}

impl DeterministicState for GasMeter {
    fn state_hash(&self) -> [u8; 32] {
        hash_bytes(&serialize_canonical(self).unwrap())
    }
}
