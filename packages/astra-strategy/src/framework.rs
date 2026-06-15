use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StrategyId {
    pub dummy: i64,
}

impl StrategyId {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StrategyState {
    pub dummy: i64,
    pub risk_gating_active: bool,
    pub active_drawdown_limit_violation: bool,
}

impl StrategyState {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StrategyDecision {
    pub dummy: i64,
    pub risk_constrained: bool,
}

impl StrategyDecision {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExecutionIntent {
    pub dummy: i64,
    pub exposure_approved: bool,
    pub margin_allocated: u64,
}

impl ExecutionIntent {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SignalWindow {
    pub dummy: i64,
}

impl SignalWindow {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StrategyCheckpoint {
    pub dummy: i64,
}

impl StrategyCheckpoint {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExecutionTrace {
    pub dummy: i64,
}

impl ExecutionTrace {
    pub fn new() -> Self {
        Default::default()
    }
}


