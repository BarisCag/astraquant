use crate::scenario::{ScenarioDefinition, ScenarioRuntime};
use astra_core::events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata};
use astra_core::serialization::serialize_canonical;
use astra_ops::control::{OperationalAction, OperationalCommand};

#[derive(Clone, Debug)]
pub struct RecoveryDrillScenario {
    pub seed: u64,
    pub venue_failure_sequence: u64,
    pub intervention_sequence: u64,
    pub target_venue: u8,
}

impl ScenarioDefinition for RecoveryDrillScenario {
    fn scenario_id(&self) -> &'static str {
        "recovery_drill"
    }

    fn get_seed(&self) -> u64 {
        self.seed
    }

    fn get_severity(&self) -> crate::scenario::ScenarioSeverity {
        crate::scenario::ScenarioSeverity {
            liquidity_drop_ppm: 0,
            latency_spike_multiplier: 1,
            collateral_haircut_ppm: 0,
            margin_requirement_multiplier: 1,
            probability_of_venue_failure_ppm: 0,
        }
    }

    fn get_activation_windows(&self) -> Vec<(u64, u64)> {
        vec![(self.intervention_sequence, self.intervention_sequence)]
    }

    fn evaluate_sequence(
        &self,
        current_sequence: u64,
        _lcg: &mut crate::lcg::DeterministicLcg,
    ) -> Vec<AstraEvent> {
        let mut injected_events = Vec::new();

        if current_sequence == self.intervention_sequence {
            let action = OperationalAction {
                operator_id: "governance_system".to_string(),
                command: OperationalCommand::ResumeVenue {
                    venue_id: self.target_venue,
                },
                sequence_applied: current_sequence,
            };

            injected_events.push(AstraEvent {
                timestamp_ns: current_sequence * 1_000_000, // synthetic
                sequence_id: current_sequence,
                event_type: EventType::OperatorAction,
                payload: serialize_canonical(&action).unwrap(),
                payload_metadata: PayloadMetadata::new(PayloadEncoding::RawBytes, 1),
            });
        }

        injected_events
    }
}
