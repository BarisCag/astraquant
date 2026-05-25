use crate::events::AstraEvent;
use crate::hashing::DeterministicState;
use crate::replay::EventReducer;

pub struct SymbolicReplayEngine;

impl SymbolicReplayEngine {
    pub fn detect_divergence<T: EventReducer + DeterministicState>(
        state_a: &mut T,
        state_b: &mut T,
        events: &[AstraEvent],
    ) -> bool {
        for event in events {
            let _ = state_a.apply(event);
            let _ = state_b.apply(event);

            if state_a.state_hash() != state_b.state_hash() {
                return true; // Divergence detected
            }
        }
        false
    }
}
