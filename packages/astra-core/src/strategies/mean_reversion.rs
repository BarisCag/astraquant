use crate::events::{AstraEvent, EventType};
use crate::hashing::{hash_bytes, DeterministicState};
use crate::marketdata::MarketTick;
use crate::serialization::{deserialize_canonical, serialize_canonical};
use crate::strategy::{Strategy, StrategyAction};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MeanReversionStrategy {
    pub id: u64,
    pub symbol: String,
    pub window_size: u64,
    pub tick_count: u64,
    pub price_sum: i64,
    pub threshold: i64,
}

impl MeanReversionStrategy {
    pub fn new(id: u64, symbol: String, window_size: u64, threshold: i64) -> Self {
        Self {
            id,
            symbol,
            window_size,
            tick_count: 0,
            price_sum: 0,
            threshold,
        }
    }
}

impl DeterministicState for MeanReversionStrategy {
    fn state_hash(&self) -> [u8; 32] {
        hash_bytes(&serialize_canonical(self).unwrap())
    }
}

impl Strategy for MeanReversionStrategy {
    fn strategy_id(&self) -> u64 {
        self.id
    }

    fn on_event(&mut self, event: &AstraEvent) -> Result<Vec<StrategyAction>, String> {
        if event.event_type == EventType::MarketTick {
            if let Ok(tick) = deserialize_canonical::<MarketTick>(&event.payload) {
                if tick.symbol == self.symbol {
                    self.tick_count += 1;
                    self.price_sum = self
                        .price_sum
                        .saturating_add(tick.bid_price.0)
                        .saturating_add(tick.ask_price.0);
                }
            }
        }
        Ok(vec![])
    }
}
