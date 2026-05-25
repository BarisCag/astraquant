use crate::events::AstraEvent;
use crate::hashing::{hash_bytes, DeterministicState};
use crate::kernel::AstraKernel;
use crate::replay::EventReducer;
use crate::vm::DeterministicVm;
use std::collections::BTreeMap;

pub struct VmOrchestrator {
    pub kernel: AstraKernel,
    pub vms: BTreeMap<u64, DeterministicVm>,
}

impl VmOrchestrator {
    pub fn new(kernel: AstraKernel) -> Self {
        Self {
            kernel,
            vms: BTreeMap::new(),
        }
    }

    pub fn register_vm(&mut self, id: u64, vm: DeterministicVm) {
        self.vms.insert(id, vm);
    }
}

impl DeterministicState for VmOrchestrator {
    fn state_hash(&self) -> [u8; 32] {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.kernel.state_hash());
        for (_, vm) in self.vms.iter() {
            bytes.extend_from_slice(&vm.state_hash());
        }
        hash_bytes(&bytes)
    }
}

impl EventReducer for VmOrchestrator {
    type Error = String;

    fn apply(&mut self, event: &AstraEvent) -> Result<(), Self::Error> {
        self.kernel.apply(event).map_err(|e| e.to_string())?;

        for (_, vm) in self.vms.iter_mut() {
            vm.run_event(event)?;
        }
        Ok(())
    }

    fn last_applied_sequence_id(&self) -> Option<u64> {
        self.kernel.last_applied_sequence_id()
    }
}
