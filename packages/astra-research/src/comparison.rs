use crate::experiment::ExperimentRun;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayDiff {
    pub base_run_id: String,
    pub target_run_id: String,
    pub sequence_divergence: u64, // First sequence where state hashes diverge
    pub event_divergence_count: u64,
    pub final_hash_match: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InterventionDriftReport {
    pub baseline_stabilization_sequence: u64,
    pub intervention_stabilization_sequence: u64,
    pub liquidity_restoration_delta_ppm: i64,
    pub contagion_suppression_score_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemicRecoveryComparison {
    pub base_recovery_score_ppm: u64,
    pub policy_recovery_score_ppm: u64,
    pub effectiveness_multiplier_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyEffectivenessReport {
    pub intervention_drift: InterventionDriftReport,
    pub systemic_recovery: SystemicRecoveryComparison,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemicDriftReport {
    pub experiment_suite_id: String,
    pub systemic_variance_score_ppm: u64,
    pub liquidity_resilience_score_ppm: u64,
    pub replay_divergence_score_ppm: u64,
    pub infrastructure_stability_score_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkDriftReport {
    pub baseline_certification_hash: [u8; 32],
    pub target_certification_hash: [u8; 32],
    pub hash_match: bool,
    pub divergence_window: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayCertificationComparison {
    pub left_run_id: String,
    pub right_run_id: String,
    pub match_status: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyTimingComparison {
    pub delay_sequences: u64,
    pub containment_loss_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemicOutcomeComparison {
    pub baseline_recovery_score_ppm: u64,
    pub target_recovery_score_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentComparison {
    pub replay_diffs: Vec<ReplayDiff>,
    pub systemic_drift: SystemicDriftReport,
    pub benchmark_drift: Option<BenchmarkDriftReport>,
}

impl ExperimentComparison {
    pub fn compare(base_run: &ExperimentRun, target_run: &ExperimentRun) -> ReplayDiff {
        ReplayDiff {
            base_run_id: base_run.run_id.clone(),
            target_run_id: target_run.run_id.clone(),
            // For now, if final hashes match, we assume sequence divergence didn't happen
            // In reality, this requires sequence-by-sequence checkpoint diffing, which requires
            // the full checkpoint lineage from astra-ops. We will stub for now.
            sequence_divergence: if base_run.final_state_hash == target_run.final_state_hash { 0 } else { 1 },
            event_divergence_count: if base_run.final_state_hash == target_run.final_state_hash { 0 } else { 1 },
            final_hash_match: base_run.final_state_hash == target_run.final_state_hash,
        }
    }

    pub fn compute_drift(runs: &[ExperimentRun]) -> SystemicDriftReport {
        let mut max_variance = 0;
        let mut min_resilience = 1_000_000;

        for run in runs {
            if run.systemic_metrics.systemic_variance_score_ppm > max_variance {
                max_variance = run.systemic_metrics.systemic_variance_score_ppm;
            }
            if run.systemic_metrics.liquidity_resilience_score_ppm < min_resilience {
                min_resilience = run.systemic_metrics.liquidity_resilience_score_ppm;
            }
        }

        SystemicDriftReport {
            experiment_suite_id: "drift_analysis".to_string(),
            systemic_variance_score_ppm: max_variance,
            liquidity_resilience_score_ppm: min_resilience,
            replay_divergence_score_ppm: 0,
            infrastructure_stability_score_ppm: 1_000_000,
        }
    }
}

// Phase 13C: Deterministic Audit Comparisons

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvariantDriftReport {
    pub baseline_violations: u64,
    pub target_violations: u64,
    pub new_violations: u64,
    pub resolved_violations: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayConsistencyComparison {
    pub left_integrity_score_ppm: u64,
    pub right_integrity_score_ppm: u64,
    pub divergence_detected: bool,
    pub first_divergent_sequence: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificationVarianceReport {
    pub left_certification_hash: [u8; 32],
    pub right_certification_hash: [u8; 32],
    pub hash_match: bool,
    pub lineage_depth_difference: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineageDeviationAnalysis {
    pub common_ancestor_sequence: u64,
    pub left_branch_depth: u64,
    pub right_branch_depth: u64,
    pub deviation_score_ppm: u64,
}

// Phase 14A: Ecology Comparisons

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BehavioralDriftReport {
    pub left_behavior_transitions: u64,
    pub right_behavior_transitions: u64,
    pub identical_transitions: u64,
    pub behavioral_drift_score_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentCascadeComparison {
    pub left_cascade_depth: u64,
    pub right_cascade_depth: u64,
    pub left_agents_affected: u64,
    pub right_agents_affected: u64,
    pub cascade_parity: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiquidityWithdrawalAnalysis {
    pub baseline_withdrawn_liquidity: u64,
    pub experiment_withdrawn_liquidity: u64,
    pub total_agents_withdrawn: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BehaviorTopologyComparison {
    pub structural_difference_score_ppm: u64,
    pub matched_edges: u64,
    pub divergent_edges: u64,
}
