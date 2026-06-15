use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ReplayThroughputReport {
    pub msgs_per_sec: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct QueuePressureBenchmark {
    pub latency_ns: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DeterministicPerfWindow {
    pub ticks: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ReplayMemoryProfile {
    pub bytes_used: u64,
}

impl ReplayThroughputReport {
    pub fn new() -> Self { Default::default() }
}
impl QueuePressureBenchmark {
    pub fn new() -> Self { Default::default() }
}
impl DeterministicPerfWindow {
    pub fn new() -> Self { Default::default() }
}
impl ReplayMemoryProfile {
    pub fn new() -> Self { Default::default() }
}
