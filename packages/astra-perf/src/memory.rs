use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DeterministicMemoryProfile {
    pub total_allocated: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AllocationPressureBoundary {
    pub max_alloc_allowed: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ReplayBufferUtilization {
    pub buffer_size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SnapshotMemoryTrace {
    pub snapshot_size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LineageMemoryFootprint {
    pub total_footprint: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MemoryParityVerifier {
    pub is_parity_valid: bool,
}

impl DeterministicMemoryProfile {
    pub fn new() -> Self {
        Default::default()
    }
}
