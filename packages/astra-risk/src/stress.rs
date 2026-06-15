use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StressScenario {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SystemicStressPropagation {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LiquidityShockBoundary {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct VolatilityStressWindow {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CascadingLiquidationTrace {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct VenueFailurePropagation {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StressReplayVerifier {}

impl StressScenario {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl SystemicStressPropagation {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl LiquidityShockBoundary {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl VolatilityStressWindow {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl CascadingLiquidationTrace {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl VenueFailurePropagation {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl StressReplayVerifier {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
