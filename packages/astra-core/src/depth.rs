use crate::types::{Price, Quantity};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepthDelta {
    pub price: Price,
    pub quantity: Quantity,
    pub is_bid: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepthSnapshot {
    pub symbol: String,
    pub bids: Vec<DepthDelta>,
    pub asks: Vec<DepthDelta>,
}

impl DepthSnapshot {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            bids: vec![],
            asks: vec![],
        }
    }

    /// Apply a depth delta to the snapshot. Routes to bids or asks based on is_bid.
    pub fn apply_delta(&mut self, delta: &DepthDelta, _update_id: u64) {
        if delta.is_bid {
            self.bids.push(delta.clone());
        } else {
            self.asks.push(delta.clone());
        }
    }
}
