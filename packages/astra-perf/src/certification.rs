use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PerformanceReplayCertificate {
    pub hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ThroughputParityProof {
    pub proof_data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MemoryEquivalenceProof {
    pub proof_data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProfilingIntegrityCertificate {
    pub is_valid: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PerformanceLineageHash {
    pub hash_data: Vec<u8>,
}

impl PerformanceReplayCertificate {
    pub fn new() -> Self {
        Default::default()
    }
}
