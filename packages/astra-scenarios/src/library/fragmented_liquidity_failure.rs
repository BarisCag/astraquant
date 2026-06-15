use crate::lcg::DeterministicLcg;
use crate::scenario::{ScenarioDefinition, ScenarioSeverity};
use astra_core::events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata};
use astra_core::serialization::serialize_canonical;

pub struct FragmentedLiquidityFailureScenario {
    pub seed: u64,
    pub severity: ScenarioSeverity,
    pub activation_windows: Vec<(u64, u64)>,
}

impl ScenarioDefinition for FragmentedLiquidityFailureScenario {
    fn scenario_id(&self) -> &'static str {
        "fragmented_liquidity_failure"
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
        lcg: &mut DeterministicLcg,
    ) -> Vec<AstraEvent> {
        let mut events = Vec::new();
        for (start, end) in &self.activation_windows {
            if current_sequence >= *start && current_sequence <= *end {
                if lcg.next_bool_ppm(10_000) {
                    events.push(AstraEvent {
                        timestamp_ns: 0,
                        sequence_id: 0,
                        event_type: EventType::VenueStatusChanged,
                        payload: serialize_canonical(&1u8).unwrap(), // VenueId 1 fails
                        payload_metadata: PayloadMetadata::new(PayloadEncoding::Bincode, 1),
                    });
                }
            }
        }
        events
    }
}
