use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ReplayPerformanceProfile {
    pub duration_ns: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExecutionThroughputWindow {
    pub total_operations: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DeterministicLatencyProfile {
    pub max_latency_ns: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MemoryPressureTrace {
    pub max_allocation_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ReplayExecutionCost {
    pub instruction_count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CertificationOverheadWindow {
    pub overhead_ns: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PerformanceLineageTrace {
    pub lineage_depth: u64,
}

impl ReplayPerformanceProfile {
    pub fn new() -> Self {
        Default::default()
    }
}
