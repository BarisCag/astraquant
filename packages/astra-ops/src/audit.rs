use crate::control::{OperationalAction, ReplayCertification};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditTrail {
    pub interventions: BTreeMap<u64, OperationalAction>,
}

impl AuditTrail {
    pub fn new() -> Self {
        Self {
            interventions: BTreeMap::new(),
        }
    }

    pub fn record(&mut self, sequence: u64, action: OperationalAction) {
        self.interventions.insert(sequence, action);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayAuditReport {
    pub certification: ReplayCertification,
    pub matches_original: bool,
    pub divergence_sequence: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointLineage {
    pub checkpoints: Vec<([u8; 32], u64)>, // Hash and sequence
}

impl CheckpointLineage {
    pub fn new() -> Self {
        Self {
            checkpoints: Vec::new(),
        }
    }

    pub fn record(&mut self, hash: [u8; 32], sequence: u64) {
        self.checkpoints.push((hash, sequence));
    }
}
