use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RiskLimit {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PositionLimitBoundary {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExposureConstraint {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DrawdownConstraint {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LeverageConstraint {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RiskViolationWindow {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ConstraintParityVerifier {}

impl RiskLimit {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl PositionLimitBoundary {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl ExposureConstraint {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl DrawdownConstraint {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl LeverageConstraint {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl RiskViolationWindow {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
impl ConstraintParityVerifier {
    pub fn dummy_method(&self) -> Self { Default::default() }
}
