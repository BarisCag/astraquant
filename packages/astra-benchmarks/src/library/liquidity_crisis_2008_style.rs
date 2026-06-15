use astra_core::events::AstraEvent;
use astra_scenarios::lcg::DeterministicLcg;
use astra_scenarios::scenario::{ExperimentParameterSet, ScenarioDefinition, ScenarioSeverity};

pub struct LiquidityCrisis2008Style {
    pub seed: u64,
}

impl ScenarioDefinition for LiquidityCrisis2008Style {
    fn scenario_id(&self) -> &'static str {
        "liquidity_crisis_2008_style"
    }

    fn get_seed(&self) -> u64 {
        self.seed
    }

    fn get_severity(&self) -> ScenarioSeverity {
        ScenarioSeverity {
            liquidity_drop_ppm: 850_000,
            latency_spike_multiplier: 10,
            margin_requirement_multiplier: 3,
            collateral_haircut_ppm: 600_000,
            probability_of_venue_failure_ppm: 200_000,
        }
    }

    fn get_activation_windows(&self) -> Vec<(u64, u64)> {
        vec![(1_000_000, 2_000_000)]
    }

    fn apply_parameters(&mut self, _params: &ExperimentParameterSet) {
        // Deterministic override via parameters
    }

    fn evaluate_sequence(
        &self,
        _current_sequence: u64,
        _lcg: &mut DeterministicLcg,
    ) -> Vec<AstraEvent> {
        Vec::new()
    }
}
