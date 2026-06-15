use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PortfolioState {
    pub dummy: i64,
}

impl PortfolioState {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PositionInventory {
    pub dummy: i64,
}

impl PositionInventory {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RealizedPnL {
    pub dummy: i64,
}

impl RealizedPnL {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UnrealizedPnL {
    pub dummy: i64,
}

impl UnrealizedPnL {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct FeeModel {
    pub dummy: i64,
}

impl FeeModel {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExecutionCostModel {
    pub dummy: i64,
}

impl ExecutionCostModel {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct InventoryExposureWindow {
    pub dummy: i64,
}

impl InventoryExposureWindow {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PortfolioCheckpoint {
    pub dummy: i64,
}

impl PortfolioCheckpoint {
    pub fn new() -> Self {
        Default::default()
    }
}
