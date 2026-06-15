use astra_core::events::AstraEvent;
use astra_scenarios::lcg::DeterministicLcg;
use astra_scenarios::scenario::{ExperimentParameterSet, ScenarioDefinition, ScenarioSeverity};

pub struct ExchangeFragmentationEvent {
    pub seed: u64,
}

impl ScenarioDefinition for ExchangeFragmentationEvent {
    fn scenario_id(&self) -> &'static str {
        "exchange_fragmentation_event"
    }

    fn get_seed(&self) -> u64 {
        self.seed
    }

    fn get_severity(&self) -> ScenarioSeverity {
        ScenarioSeverity {
            liquidity_drop_ppm: 300_000,
            latency_spike_multiplier: 50,
            margin_requirement_multiplier: 1,
            collateral_haircut_ppm: 0,
            probability_of_venue_failure_ppm: 800_000,
        }
    }

    fn get_activation_windows(&self) -> Vec<(u64, u64)> {
        vec![(2_000_000, 3_000_000)]
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
