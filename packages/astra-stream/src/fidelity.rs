use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MarketMicrostructureProfile {
    pub _dummy: u64,
}

impl MarketMicrostructureProfile {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SpreadRegimeWindow {
    pub _dummy: u64,
}

impl SpreadRegimeWindow {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LiquidityRegimeBoundary {
    pub _dummy: u64,
}

impl LiquidityRegimeBoundary {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct VolatilityRegimeTrace {
    pub _dummy: u64,
}

impl VolatilityRegimeTrace {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct OrderFlowPressureProfile {
    pub _dummy: u64,
}

impl OrderFlowPressureProfile {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct VenueFragmentationSnapshot {
    pub _dummy: u64,
}

impl VenueFragmentationSnapshot {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MicrostructureReplayVerifier {
    pub _dummy: u64,
}

impl MicrostructureReplayVerifier {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}
