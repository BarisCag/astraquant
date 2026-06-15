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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventType, PayloadEncoding, PayloadMetadata};
    use crate::exchange::ExchangeRuntime;
    use crate::kernel::AstraKernel;
    use crate::risk::create_default_risk_engine;
    use crate::runtime::StrategyRuntime;
    use crate::types::{Money, Quantity};

    #[test]
    fn test_symbolic_divergence_detection() {
        let limits = create_default_risk_engine(Money::new(1000), Quantity::new(10));
        let mut kernel_a =
            AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits.clone())));
        let mut kernel_b = AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits)));

        // Create some events
        let event1 = AstraEvent::new(
            1,
            1,
            EventType::MarketTick,
            vec![],
            PayloadMetadata::new(PayloadEncoding::RawBytes, 1),
        );

        // Mutate one kernel's metrics directly to force divergence that survives apply()
        kernel_a.metrics.error_count = 999;

        // Ensure detect_divergence catches it
        let events = vec![event1];
        let diverged =
            SymbolicReplayEngine::detect_divergence(&mut kernel_a, &mut kernel_b, &events);
        assert!(
            diverged,
            "SymbolicReplayEngine failed to detect divergence between two distinct kernel states"
        );
    }
}
