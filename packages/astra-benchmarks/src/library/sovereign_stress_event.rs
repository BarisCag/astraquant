use astra_core::events::AstraEvent;
use astra_scenarios::lcg::DeterministicLcg;
use astra_scenarios::scenario::{ExperimentParameterSet, ScenarioDefinition, ScenarioSeverity};

pub struct SovereignStressEvent {
    pub seed: u64,
}

impl ScenarioDefinition for SovereignStressEvent {
    fn scenario_id(&self) -> &'static str {
        "sovereign_stress_event"
    }

    fn get_seed(&self) -> u64 {
        self.seed
    }

    fn get_severity(&self) -> ScenarioSeverity {
        ScenarioSeverity {
            liquidity_drop_ppm: 600_000,
            latency_spike_multiplier: 1,
            margin_requirement_multiplier: 10,
            collateral_haircut_ppm: 950_000,
            probability_of_venue_failure_ppm: 0,
        }
    }

    fn get_activation_windows(&self) -> Vec<(u64, u64)> {
        vec![(1_000_000, 5_000_000)]
    }

    fn apply_parameters(&mut self, _params: &ExperimentParameterSet) {}

    fn evaluate_sequence(
        &self,
        _current_sequence: u64,
        _lcg: &mut DeterministicLcg,
    ) -> Vec<AstraEvent> {
        Vec::new()
    }
}
