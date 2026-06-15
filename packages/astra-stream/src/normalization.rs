use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CanonicalTimestamp {
    pub _dummy: u64,
}

impl CanonicalTimestamp {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NormalizedTrade {
    pub _dummy: u64,
}

impl NormalizedTrade {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NormalizedQuote {
    pub _dummy: u64,
}

impl NormalizedQuote {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SessionNormalizationManifest {
    pub _dummy: u64,
}

impl SessionNormalizationManifest {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TimestampAlignmentWindow {
    pub _dummy: u64,
}

impl TimestampAlignmentWindow {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct VenueSequenceNormalizer {
    pub _dummy: u64,
}

impl VenueSequenceNormalizer {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}
