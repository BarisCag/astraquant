use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExecutionQualityReport {
    pub dummy: i64,
}

impl ExecutionQualityReport {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct FillEfficiencyWindow {
    pub dummy: i64,
}

impl FillEfficiencyWindow {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SlippageAttribution {
    pub dummy: i64,
}

impl SlippageAttribution {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LatencyImpactAnalysis {
    pub dummy: i64,
}

impl LatencyImpactAnalysis {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct InventoryPressureAnalysis {
    pub dummy: i64,
}

impl InventoryPressureAnalysis {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TradeLifecycleTrace {
    pub dummy: i64,
}

impl TradeLifecycleTrace {
    pub fn new() -> Self {
        Default::default()
    }
}
