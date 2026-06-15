use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InvariantSeverity {
    Critical,
    Warning,
    Advisory,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InvariantCategory {
    SequenceMonotonicity,
    JournalContinuity,
    ReplayParity,
    LiquidityConservation,
    SettlementConservation,
    CollateralIntegrity,
    MarginConsistency,
    CheckpointLineageContinuity,
    PolicyBoundaryValidity,
    VenueTopologyConsistency,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvariantDefinition {
    pub invariant_id: u64,
    pub category: InvariantCategory,
    pub severity: InvariantSeverity,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvariantViolation {
    pub invariant_id: u64,
    pub category: InvariantCategory,
    pub severity: InvariantSeverity,
    pub detected_at_sequence: u64,
    pub expected_hash: [u8; 32],
    pub actual_hash: [u8; 32],
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvariantEvaluationWindow {
    pub start_sequence: u64,
    pub end_sequence: u64,
    pub violations: Vec<InvariantViolation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvariantRegistry {
    pub invariants: Vec<InvariantDefinition>,
}

impl InvariantRegistry {
    pub fn new() -> Self {
        Self {
            invariants: Vec::new(),
        }
    }

    pub fn register(&mut self, def: InvariantDefinition) {
        self.invariants.push(def);
    }

    pub fn evaluate_sequence_monotonicity(&self, sequences: &[u64]) -> Option<InvariantViolation> {
        for i in 1..sequences.len() {
            if sequences[i] <= sequences[i - 1] {
                return Some(InvariantViolation {
                    invariant_id: 0,
                    category: InvariantCategory::SequenceMonotonicity,
                    severity: InvariantSeverity::Critical,
                    detected_at_sequence: sequences[i],
                    expected_hash: [0u8; 32],
                    actual_hash: [0u8; 32],
                    description: format!(
                        "Sequence monotonicity violation: sequence {} at index {} is not greater than previous sequence {}",
                        sequences[i], i, sequences[i - 1]
                    ),
                });
            }
        }
        None
    }
}
