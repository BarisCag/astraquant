use crate::hashing::{hash_bytes, DeterministicState};
use crate::marketdata::MarketTick;
use crate::serialization::serialize_canonical;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoricalFeed {
    pub ticks: Vec<MarketTick>,
    pub cursor: usize,
}

impl HistoricalFeed {
    pub fn new(ticks: Vec<MarketTick>) -> Self {
        Self { ticks, cursor: 0 }
    }

    pub fn next_tick(&mut self) -> Option<&MarketTick> {
        if self.cursor < self.ticks.len() {
            let tick = &self.ticks[self.cursor];
            self.cursor += 1;
            Some(tick)
        } else {
            None
        }
    }
}

impl DeterministicState for HistoricalFeed {
    fn state_hash(&self) -> [u8; 32] {
        hash_bytes(&serialize_canonical(self).expect("HistoricalFeed serialization failed"))
    }
}
