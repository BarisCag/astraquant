use astra_core::events::AstraEvent;

pub struct ScenarioEventInjector;

impl ScenarioEventInjector {
    pub fn inject_events(events: Vec<AstraEvent>, current_sequence: u64) -> Vec<AstraEvent> {
        events.into_iter().map(|mut e| {
            e.sequence_id = current_sequence;
            e
        }).collect()
    }
}
