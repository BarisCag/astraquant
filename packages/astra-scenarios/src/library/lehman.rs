use astra_core::events::{AstraEvent, EventType};
use crate::scenario::{ScenarioDefinition, ScenarioSeverity, ExperimentParameterSet};
use astra_policy::policy::{PolicyAction, PolicyDirective, PolicySeverityTier, PolicyExecutionWindow};
use bincode::Options;

pub struct LehmanLiquiditySupportScenario {
    pub seed: u64,
    pub shock_sequence: u64,
    pub intervention_sequence: u64,
    pub support_amount: u64,
}

impl ScenarioDefinition for LehmanLiquiditySupportScenario {
    fn scenario_id(&self) -> &'static str {
        "lehman_liquidity_support"
    }

    fn get_seed(&self) -> u64 {
        self.seed
    }

    fn get_severity(&self) -> ScenarioSeverity {
        ScenarioSeverity {
            liquidity_drop_ppm: 900_000,
            latency_spike_multiplier: 5,
            margin_requirement_multiplier: 2,
            collateral_haircut_ppm: 500_000,
            probability_of_venue_failure_ppm: 10_000,
        }
    }

    fn get_activation_windows(&self) -> Vec<(u64, u64)> {
        vec![(self.shock_sequence, self.intervention_sequence + 100)]
    }

    fn apply_parameters(&mut self, params: &ExperimentParameterSet) {
        if params.liquidity_drop_ppm > 0 {
            // Can override severity
        }
        if params.recovery_delay_sequences > 0 {
            self.intervention_sequence = self.shock_sequence + params.recovery_delay_sequences;
        }
    }

    fn evaluate_sequence(&self, current_sequence: u64, _lcg: &mut crate::lcg::DeterministicLcg) -> Vec<AstraEvent> {
        let mut events = Vec::new();

        if current_sequence == self.shock_sequence {
            // Emit a market stress event
            let payload = bincode::options().with_little_endian().with_fixint_encoding()
                .serialize(&astra_router::failure::VenueFailureEvent::LiquidityCollapse {
                    venue_id: astra_router::venue::VenueId(1),
                    sequence_id: current_sequence,
                    symbol: "AAPL".to_string(),
                    fraction_to_remove: 90,
                }).unwrap();

            events.push(AstraEvent::new_raw(
                current_sequence * 1_000_000,
                current_sequence,
                EventType::MarketStressInjected,
                payload,
            ));
        }

        if current_sequence == self.intervention_sequence {
            // Central Bank steps in
            let action = PolicyAction::LiquidityInjection { amount: self.support_amount };
            let directive = PolicyDirective {
                directive_id: 1,
                action,
                severity: PolicySeverityTier::Emergency,
                execution_window: PolicyExecutionWindow {
                    start_sequence: current_sequence,
                    end_sequence: current_sequence + 50,
                },
            };

            let payload = bincode::options().with_little_endian().with_fixint_encoding()
                .serialize(&directive).unwrap();

            events.push(AstraEvent::new_raw(
                current_sequence * 1_000_000,
                current_sequence,
                EventType::PolicyAction,
                payload,
            ));
        }

        events
    }
}
