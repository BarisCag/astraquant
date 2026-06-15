use astra_scenarios::scenario::{ScenarioDefinition, ScenarioSeverity, ExperimentParameterSet};
use astra_core::events::AstraEvent;
use astra_scenarios::lcg::DeterministicLcg;

pub struct RepoMarketSeizure {
    pub seed: u64,
}

impl ScenarioDefinition for RepoMarketSeizure {
    fn scenario_id(&self) -> &'static str {
        "repo_market_seizure"
    }

    fn get_seed(&self) -> u64 {
        self.seed
    }

    fn get_severity(&self) -> ScenarioSeverity {
        ScenarioSeverity {
            liquidity_drop_ppm: 500_000,
            latency_spike_multiplier: 2,
            margin_requirement_multiplier: 5,
            collateral_haircut_ppm: 900_000,
            probability_of_venue_failure_ppm: 50_000,
        }
    }

    fn get_activation_windows(&self) -> Vec<(u64, u64)> {
        vec![(500_000, 1_500_000)]
    }

    fn apply_parameters(&mut self, _params: &ExperimentParameterSet) {
    }

    fn evaluate_sequence(&self, _current_sequence: u64, _lcg: &mut DeterministicLcg) -> Vec<AstraEvent> {
        Vec::new()
    }
}
