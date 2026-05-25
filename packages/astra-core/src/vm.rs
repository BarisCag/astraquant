use crate::events::AstraEvent;
use crate::hashing::{hash_bytes, DeterministicState};
use crate::package::StrategyPackage;
use crate::sandbox::WasmSandbox;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct DeterministicVm {
    pub package: StrategyPackage,
    pub sandbox: WasmSandbox,
}

impl DeterministicVm {
    pub fn load(package: StrategyPackage, gas_limit: u64) -> Result<Self, String> {
        if !package.verify() {
            return Err("Corrupted package checksum".to_string());
        }
        Ok(Self {
            package,
            sandbox: WasmSandbox::new(gas_limit),
        })
    }

    pub fn run_event(&mut self, event: &AstraEvent) -> Result<(), String> {
        self.sandbox.execute_event(event)
    }
}

impl DeterministicState for DeterministicVm {
    fn state_hash(&self) -> [u8; 32] {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.package.state_hash());
        bytes.extend_from_slice(&self.sandbox.state_hash());
        hash_bytes(&bytes)
    }
}
