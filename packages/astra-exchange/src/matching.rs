use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct QueuePosition {
    pub position: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExecutionPriority {
    pub priority: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct OrderQueueSnapshot {
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DeterministicFillChain {
    pub fills: Vec<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExecutionLineageWindow {
    pub start: u64,
    pub end: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MatchingDecisionTrace {
    pub trace_id: u64,
}

impl QueuePosition {
    pub fn new() -> Self { Default::default() }
}
impl ExecutionPriority {
    pub fn new() -> Self { Default::default() }
}
impl OrderQueueSnapshot {
    pub fn new() -> Self { Default::default() }
}
impl DeterministicFillChain {
    pub fn new() -> Self { Default::default() }
}
impl ExecutionLineageWindow {
    pub fn new() -> Self { Default::default() }
}
impl MatchingDecisionTrace {
    pub fn new() -> Self { Default::default() }
}
