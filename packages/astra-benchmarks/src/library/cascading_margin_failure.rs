use astra_scenarios::scenario::{ScenarioDefinition, ScenarioSeverity, ExperimentParameterSet};
use astra_core::events::AstraEvent;
use astra_scenarios::lcg::DeterministicLcg;

pub struct CascadingMarginFailure {
    pub seed: u64,
}

impl ScenarioDefinition for CascadingMarginFailure {
    fn scenario_id(&self) -> &'static str {
        "cascading_margin_failure"
    }

    fn get_seed(&self) -> u64 {
        self.seed
    }

    fn get_severity(&self) -> ScenarioSeverity {
        ScenarioSeverity {
            liquidity_drop_ppm: 400_000,
            latency_spike_multiplier: 1,
            margin_requirement_multiplier: 8,
            collateral_haircut_ppm: 400_000,
            probability_of_venue_failure_ppm: 0,
        }
    }

    fn get_activation_windows(&self) -> Vec<(u64, u64)> {
        vec![(1_000_000, 2_000_000)]
    }

    fn apply_parameters(&mut self, _params: &ExperimentParameterSet) {
    }

    fn evaluate_sequence(&self, _current_sequence: u64, _lcg: &mut DeterministicLcg) -> Vec<AstraEvent> {
        Vec::new()
    }
}
