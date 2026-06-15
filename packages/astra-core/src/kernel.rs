//! The deterministic sandbox execution environment.
//!
//! The `AstraKernel` is the pure, mathematical core of the operating system.
//! It contains no asynchronous primitives, no networking, and no wall-clock I/O.
//! It acts purely as a deterministic `EventReducer` that processes a stream
//! of ordered events and produces cryptographically verifiable state hashes.

use crate::events::AstraEvent;
use crate::hashing::{hash_bytes, DeterministicState};
use crate::metrics::KernelMetrics;
use crate::proof::StateTransitionProof;
use crate::replay::EventReducer;
use crate::runtime::StrategyRuntime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AstraKernel {
    pub sequence_id: u64,
    pub strategy_runtime: StrategyRuntime,
    pub metrics: KernelMetrics,
    pub last_proof: Option<StateTransitionProof>,
    pub yield_to_rl: bool,
}

impl AstraKernel {
    pub fn new(strategy_runtime: StrategyRuntime) -> Self {
        Self {
            sequence_id: 0,
            strategy_runtime,
            metrics: KernelMetrics::new(),
            last_proof: None,
            yield_to_rl: false,
        }
    }
}

impl EventReducer for AstraKernel {
    type Error = String;
    fn apply(&mut self, event: &AstraEvent) -> Result<(), Self::Error> {
        let pre_state = self.state_hash();
        
        self.sequence_id = event.sequence_id;
        let result = self.strategy_runtime.apply(event);
        if result.is_err() {
            self.metrics.error_count += 1;
        } else {
            self.metrics.total_events_processed += 1;
            if event.event_type == crate::events::EventType::MarketTick {
                self.yield_to_rl = true;
            }
        }
        
        let post_state = self.state_hash();
        self.last_proof = Some(StateTransitionProof::generate(pre_state, event, post_state));
        
        result
    }
    fn last_applied_sequence_id(&self) -> Option<u64> {
        Some(self.sequence_id)
    }
}

impl DeterministicState for AstraKernel {
    fn state_hash(&self) -> [u8; 32] {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.sequence_id.to_le_bytes());
        bytes.extend_from_slice(&self.strategy_runtime.state_hash());
        bytes.extend_from_slice(&self.metrics.state_hash());
        hash_bytes(&bytes)
    }
}


