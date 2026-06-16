use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyParityReport {
    pub identical_actions: u64,
    pub divergent_actions: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BehavioralPolicyComparison {
    pub left_policy_hash: [u8; 32],
    pub right_policy_hash: [u8; 32],
    pub difference_score_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemicPolicyImpactStudy {
    pub baseline_recovery_sequences: u64,
    pub policy_recovery_sequences: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyEvaluationRun {
    pub run_id: String,
    pub parity: PolicyParityReport,
    pub impact: SystemicPolicyImpactStudy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OfflineReplayEvaluator {}

impl Default for OfflineReplayEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl OfflineReplayEvaluator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate(&self) -> PolicyEvaluationRun {
        PolicyEvaluationRun {
            run_id: String::new(),
            parity: PolicyParityReport {
                identical_actions: 0,
                divergent_actions: 0,
            },
            impact: SystemicPolicyImpactStudy {
                baseline_recovery_sequences: 0,
                policy_recovery_sequences: 0,
            },
        }
    }
}
