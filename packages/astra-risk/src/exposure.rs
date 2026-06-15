use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExposureWindow {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PortfolioExposureProfile {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct VenueExposureBoundary {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AssetConcentrationMap {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CounterpartyExposureTrace {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct InventoryRiskSurface {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExposureLineageTrace {}

impl ExposureWindow {
    pub fn dummy_method(&self) -> Self {
        Default::default()
    }
}
impl PortfolioExposureProfile {
    pub fn dummy_method(&self) -> Self {
        Default::default()
    }
}
impl VenueExposureBoundary {
    pub fn dummy_method(&self) -> Self {
        Default::default()
    }
}
impl AssetConcentrationMap {
    pub fn dummy_method(&self) -> Self {
        Default::default()
    }
}
impl CounterpartyExposureTrace {
    pub fn dummy_method(&self) -> Self {
        Default::default()
    }
}
impl InventoryRiskSurface {
    pub fn dummy_method(&self) -> Self {
        Default::default()
    }
}
impl ExposureLineageTrace {
    pub fn dummy_method(&self) -> Self {
        Default::default()
    }
}
