use serde::{Deserialize, Serialize};

use astra_scenarios::scenario::ExperimentParameterSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentDefinition {
    pub experiment_id: String,
    pub base_scenario: String,
    pub deterministic_seed: u64,
    pub max_sequences: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemicMetrics {
    pub experiment_replay_integrity_ppm: u64,
    pub systemic_variance_score_ppm: u64,
    pub liquidity_resilience_score_ppm: u64,
    pub recovery_success_score_ppm: u64,
    pub infrastructure_stability_score_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentRun {
    pub run_id: String,
    pub parameter_set: ExperimentParameterSet,
    pub initial_seed: u64,
    pub final_state_hash: String,
    pub replay_certification_hash: String,
    pub systemic_metrics: SystemicMetrics,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentSuite {
    pub suite_id: String,
    pub definition: ExperimentDefinition,
    pub runs: Vec<ExperimentRun>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentCheckpoint {
    pub checkpoint_hash: String,
    pub sequence_id: u64,
    pub scenario_state_hash: String,
    pub exchange_state_hash: String,
    pub replay_parent_hash: String,
}
