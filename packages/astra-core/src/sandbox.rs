use crate::abi::VmStateDump;
use crate::events::AstraEvent;
use crate::gas::GasMeter;
use crate::hashing::{hash_bytes, DeterministicState};
use crate::serialization::serialize_canonical;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WasmSandbox {
    pub memory: Vec<u8>,
    pub gas_meter: GasMeter,
    pub state_dump: VmStateDump,
    pub emitted_events: Vec<AstraEvent>,
}

impl WasmSandbox {
    pub fn new(gas_limit: u64) -> Self {
        Self {
            memory: Vec::new(),
            gas_meter: GasMeter::new(gas_limit),
            state_dump: VmStateDump {
                memory_hash: [0; 32],
                execution_pointer: 0,
            },
            emitted_events: Vec::new(),
        }
    }

    pub fn execute_event(&mut self, event: &AstraEvent) -> Result<(), String> {
        // Gas accounting only; no WASM bytecode interpreter is linked in this crate.
        self.gas_meter.consume(100)?;
        self.state_dump.execution_pointer += 1;
        self.state_dump.memory_hash = hash_bytes(&serialize_canonical(event).unwrap());
        Ok(())
    }
}

impl DeterministicState for WasmSandbox {
    fn state_hash(&self) -> [u8; 32] {
        hash_bytes(&serialize_canonical(self).unwrap())
    }
}
