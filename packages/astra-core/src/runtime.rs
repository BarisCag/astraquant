//! Strategy runtime layered on top of the exchange reducer.

use crate::events::AstraEvent;
use crate::exchange::ExchangeRuntime;
use crate::hashing::{hash_bytes, DeterministicState};
use crate::replay::EventReducer;
use crate::strategies::mean_reversion::MeanReversionStrategy;
use crate::strategy::Strategy;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum AnyStrategy {
    MeanReversion(MeanReversionStrategy),
}

impl AnyStrategy {
    fn state_hash(&self) -> [u8; 32] {
        match self {
            Self::MeanReversion(s) => s.state_hash(),
        }
    }

    fn on_event(&mut self, event: &AstraEvent) -> Result<(), String> {
        match self {
            Self::MeanReversion(s) => {
                let _actions = s.on_event(event)?;
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrategyRuntime {
    pub exchange: ExchangeRuntime,
    pub strategies: BTreeMap<u64, AnyStrategy>,
}

impl StrategyRuntime {
    pub fn new(exchange: ExchangeRuntime) -> Self {
        Self {
            exchange,
            strategies: BTreeMap::new(),
        }
    }

    pub fn add_strategy(&mut self, strategy: AnyStrategy) {
        let id = match &strategy {
            AnyStrategy::MeanReversion(s) => s.strategy_id(),
        };
        self.strategies.insert(id, strategy);
    }
}

impl EventReducer for StrategyRuntime {
    type Error = String;

    fn apply(&mut self, event: &AstraEvent) -> Result<(), Self::Error> {
        self.exchange.apply(event)?;
        for strategy in self.strategies.values_mut() {
            strategy.on_event(event)?;
        }
        Ok(())
    }

    fn last_applied_sequence_id(&self) -> Option<u64> {
        self.exchange.last_applied_sequence_id()
    }
}

impl DeterministicState for StrategyRuntime {
    fn state_hash(&self) -> [u8; 32] {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.exchange.state_hash());
        for id in self.strategies.keys() {
            bytes.extend_from_slice(&id.to_le_bytes());
            if let Some(strategy) = self.strategies.get(id) {
                bytes.extend_from_slice(&strategy.state_hash());
            }
        }
        hash_bytes(&bytes)
    }
}
