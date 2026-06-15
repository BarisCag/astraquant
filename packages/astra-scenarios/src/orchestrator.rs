use astra_exchange::runtime::ExchangeRuntime;
use astra_core::events::AstraEvent;
use crate::scenario::{ScenarioDefinition, ScenarioRuntime};
use crate::lcg::DeterministicLcg;
use crate::injector::ScenarioEventInjector;
use crate::checkpoint::ScenarioCheckpoint;

pub struct ScenarioOrchestrator {
    pub exchange: ExchangeRuntime,
    pub scenario_runtime: ScenarioRuntime,
    pub lcg: DeterministicLcg,
}

impl ScenarioOrchestrator {
    pub fn new(exchange: ExchangeRuntime, scenario_id: String, seed: u64) -> Self {
        Self {
            exchange,
            scenario_runtime: ScenarioRuntime::new(scenario_id, seed),
            lcg: DeterministicLcg::new(seed),
        }
    }

    pub fn step(&mut self, base_event: AstraEvent, active_scenario: Option<&dyn ScenarioDefinition>) -> Vec<AstraEvent> {
        self.scenario_runtime.advance();
        let current_sequence = self.scenario_runtime.current_sequence;

        let mut output_events = Vec::new();

        // 1. Process base event
        if let Ok(mut reactive_events) = self.exchange.apply_event(&base_event) {
            output_events.append(&mut reactive_events);
        }

        // 2. Inject Scenario Stress Events
        if let Some(scenario) = active_scenario {
            let injected_events = scenario.evaluate_sequence(current_sequence, &mut self.lcg);
            let sequenced_events = ScenarioEventInjector::inject_events(injected_events, current_sequence);

            for injected in sequenced_events {
                // We apply injected events to the exchange.
                // Injected events could be VenueStatusChanged, MarginCallIssued, etc.
                if let Ok(mut reactive) = self.exchange.apply_event(&injected) {
                    output_events.push(injected);
                    output_events.append(&mut reactive);
                }
            }
        }

        output_events
    }

    pub fn create_checkpoint(&self) -> ScenarioCheckpoint {
        let exchange_hash = self.exchange.generate_global_hash();
        ScenarioCheckpoint::new(self.scenario_runtime.clone(), exchange_hash)
    }
}
