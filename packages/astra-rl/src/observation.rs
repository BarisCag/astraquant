use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObservationVector {
    pub features: BTreeMap<String, i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObservationWindow {
    pub sequence: u64,
    pub observations: Vec<ObservationVector>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObservationHash {
    pub hash: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayObservationDataset {
    pub windows: Vec<ObservationWindow>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanonicalFeatureExtractor {}

impl Default for CanonicalFeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl CanonicalFeatureExtractor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn extract(&self, _sequence: u64) -> ObservationVector {
        ObservationVector {
            features: BTreeMap::new(),
        }
    }
}
