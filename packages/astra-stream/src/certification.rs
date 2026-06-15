use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ReplayFidelityCertificate {
    pub _dummy: u64,
}

impl ReplayFidelityCertificate {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct HistoricalParityProof {
    pub _dummy: u64,
}

impl HistoricalParityProof {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MarketReplayCertification {
    pub _dummy: u64,
}

impl MarketReplayCertification {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ReplayIntegrityWindow {
    pub _dummy: u64,
}

impl ReplayIntegrityWindow {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SessionEquivalenceProof {
    pub _dummy: u64,
}

impl SessionEquivalenceProof {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct HistoricalLineageHash {
    pub _dummy: u64,
}

impl HistoricalLineageHash {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}
