use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct OrderBookReplayVerifier {
    pub verified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SnapshotParityValidator {
    pub valid: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct FillReplayCertification {
    pub cert_id: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExecutionStateProof {
    pub proof: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct QueueStateHash {
    pub hash: [u8; 32],
}

impl OrderBookReplayVerifier {
    pub fn new() -> Self {
        Default::default()
    }
}
impl SnapshotParityValidator {
    pub fn new() -> Self {
        Default::default()
    }
}
impl FillReplayCertification {
    pub fn new() -> Self {
        Default::default()
    }
}
impl ExecutionStateProof {
    pub fn new() -> Self {
        Default::default()
    }
}
impl QueueStateHash {
    pub fn new() -> Self {
        Default::default()
    }
}
