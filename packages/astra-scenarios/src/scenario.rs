use astra_core::events::AstraEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ExperimentParameterSet {
    pub liquidity_drop_ppm: u64,
    pub margin_multiplier_ppm: u64,
    pub venue_latency_sequences: u64,
    pub collateral_haircut_ppm: u64,
    pub stress_severity_ppm: u64,
    pub recovery_delay_sequences: u64,
}

impl Default for ExperimentParameterSet {
    fn default() -> Self {
        Self {
            liquidity_drop_ppm: 0,
            margin_multiplier_ppm: 1_000_000,
            venue_latency_sequences: 0,
            collateral_haircut_ppm: 0,
            stress_severity_ppm: 0,
            recovery_delay_sequences: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioSeverity {
    pub liquidity_drop_ppm: u64,
    pub latency_spike_multiplier: u64,
    pub collateral_haircut_ppm: u64,
    pub margin_requirement_multiplier: u64,
    pub probability_of_venue_failure_ppm: u64,
}

pub trait ScenarioDefinition {
    fn scenario_id(&self) -> &'static str;
    fn get_seed(&self) -> u64;
    fn get_severity(&self) -> ScenarioSeverity;
    fn get_activation_windows(&self) -> Vec<(u64, u64)>; // start_seq, end_seq

    fn apply_parameters(&mut self, params: &ExperimentParameterSet) {
        let _ = params;
    }

    /// Given the current sequence and LCG state, potentially generate injected events
    fn evaluate_sequence(
        &self,
        current_sequence: u64,
        lcg: &mut crate::lcg::DeterministicLcg,
    ) -> Vec<AstraEvent>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioRuntime {
    pub scenario_id: String,
    pub current_sequence: u64,
    pub seed: u64,
    pub active: bool,
}

impl ScenarioRuntime {
    pub fn new(scenario_id: String, seed: u64) -> Self {
        Self {
            scenario_id,
            current_sequence: 0,
            seed,
            active: true,
        }
    }

    pub fn advance(&mut self) {
        self.current_sequence += 1;
    }
}
