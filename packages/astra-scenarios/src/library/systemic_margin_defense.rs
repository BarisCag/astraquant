use crate::lcg::DeterministicLcg;
use crate::scenario::{ScenarioDefinition, ScenarioSeverity};
use astra_core::events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata};
use astra_core::serialization::serialize_canonical;

pub struct SystemicMarginDefenseScenario {
    pub seed: u64,
    pub severity: ScenarioSeverity,
    pub activation_windows: Vec<(u64, u64)>,
}

impl ScenarioDefinition for SystemicMarginDefenseScenario {
    fn scenario_id(&self) -> &'static str {
        "systemic_margin_defense"
    }

    fn get_seed(&self) -> u64 {
        self.seed
    }

    fn get_severity(&self) -> ScenarioSeverity {
        self.severity.clone()
    }

    fn get_activation_windows(&self) -> Vec<(u64, u64)> {
        self.activation_windows.clone()
    }

    fn evaluate_sequence(
        &self,
        current_sequence: u64,
        _lcg: &mut DeterministicLcg,
    ) -> Vec<AstraEvent> {
        let mut events = Vec::new();
        for (start, _end) in &self.activation_windows {
            if current_sequence == *start {
                events.push(AstraEvent {
                    timestamp_ns: 0,
                    sequence_id: 0,
                    event_type: EventType::SystemicCascadeTriggered,
                    payload: serialize_canonical(&"SystemicMarginDefense").unwrap(),
                    payload_metadata: PayloadMetadata::new(PayloadEncoding::Bincode, 1),
                });
            }
        }
        events
    }
}
