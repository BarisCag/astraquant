use serde::{Deserialize, Serialize};
use crate::benchmark::BenchmarkManifest;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResearchStudy {
    pub study_id: String,
    pub title: String,
    pub benchmark_manifest: BenchmarkManifest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyComparisonStudy {
    pub study_id: String,
    pub control_run_id: String,
    pub variant_run_id: String,
    pub variance_score_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemicRecoveryStudy {
    pub study_id: String,
    pub recovery_duration_sequences: u64,
    pub containment_efficiency_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InterventionTimingStudy {
    pub study_id: String,
    pub baseline_loss: u64,
    pub delayed_loss: u64,
    pub immediate_loss: u64,
}

// Phase 14B: Deterministic RL Benchmarking

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RLBenchmarkStudy {
    pub study_id: String,
    pub dataset_hash: [u8; 32],
    pub evaluation_run_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyLearningComparison {
    pub baseline_policy_hash: [u8; 32],
    pub learned_policy_hash: [u8; 32],
    pub performance_delta_ppm: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdaptiveContainmentStudy {
    pub study_id: String,
    pub containment_efficiency_ppm: u64,
    pub total_interventions: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BehaviorOptimizationReport {
    pub study_id: String,
    pub behavior_stability_improvement_ppm: i64,
    pub reward_efficiency_improvement_ppm: i64,
}

// Phase 15A: Distributed Execution Benchmarking

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DistributedBenchmarkStudy {
    pub study_id: String,
    pub cluster_manifest_hash: [u8; 32],
    pub total_shards_evaluated: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayClusterStudy {
    pub study_id: String,
    pub node_count: u64,
    pub replay_acceleration_factor_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificationFanoutAnalysis {
    pub study_id: String,
    pub shard_parity_matches: u64,
    pub total_certifications_merged: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DistributedSystemicStudy {
    pub study_id: String,
    pub global_lineage_hash: [u8; 32],
    pub distributed_parity_verified: bool,
}

// Phase 16A: Formal Determinism Benchmarking

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormalDeterminismStudy {
    pub study_id: String,
    pub proof_hash: [u8; 32],
    pub is_formally_verified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayEquivalenceStudy {
    pub study_id: String,
    pub baseline_proof_hash: [u8; 32],
    pub comparison_proof_hash: [u8; 32],
    pub equivalence_matched: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DistributedParityStudy {
    pub study_id: String,
    pub monolithic_terminal_hash: [u8; 32],
    pub distributed_terminal_hash: [u8; 32],
    pub parity_verified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificationIntegrityStudy {
    pub study_id: String,
    pub total_certifications: u64,
    pub chain_integrity_score_ppm: u64,
}

// Phase 17A: Federated Deterministic Benchmarking

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FederatedReplayStudy {
    pub study_id: String,
    pub active_federations: u64,
    pub global_treaty_matched: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SovereignSystemicStudy {
    pub study_id: String,
    pub sovereign_domain_hash: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrossClusterContainmentStudy {
    pub study_id: String,
    pub containment_efficiency_score_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FederationParityAnalysis {
    pub study_id: String,
    pub total_parity_agreements: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayTreatyStressStudy {
    pub study_id: String,
    pub treaty_compliance_score_ppm: u64,
}
