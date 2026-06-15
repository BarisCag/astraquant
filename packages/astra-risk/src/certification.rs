use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RiskReplayCertification {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExposureParityProof {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MarginEquivalenceProof {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StressIntegrityCertificate {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RiskLineageHash {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RiskStateProof {}

impl RiskReplayCertification {
    pub fn dummy_method(&self) -> Self {
        Default::default()
    }
}
impl ExposureParityProof {
    pub fn dummy_method(&self) -> Self {
        Default::default()
    }
}
impl MarginEquivalenceProof {
    pub fn dummy_method(&self) -> Self {
        Default::default()
    }
}
impl StressIntegrityCertificate {
    pub fn dummy_method(&self) -> Self {
        Default::default()
    }
}
impl RiskLineageHash {
    pub fn dummy_method(&self) -> Self {
        Default::default()
    }
}
impl RiskStateProof {
    pub fn dummy_method(&self) -> Self {
        Default::default()
    }
}
