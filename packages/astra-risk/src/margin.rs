use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MarginRequirement {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MaintenanceMarginBoundary {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LiquidationThreshold {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CollateralCoverageWindow {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DeterministicLiquidationModel {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MarginStressProfile {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MarginParityVerifier {}

impl MarginRequirement {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl MaintenanceMarginBoundary {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl LiquidationThreshold {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl CollateralCoverageWindow {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl DeterministicLiquidationModel {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl MarginStressProfile {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl MarginParityVerifier {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
