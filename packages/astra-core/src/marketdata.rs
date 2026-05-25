use crate::hashing::{hash_bytes, DeterministicState};
use crate::serialization::serialize_canonical;
use crate::types::{Price, Quantity};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketTick {
    pub symbol: String,
    pub timestamp_ns: u64,
    pub bid_price: Price,
    pub ask_price: Price,
    pub bid_quantity: Quantity,
    pub ask_quantity: Quantity,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OHLCVBar {
    pub symbol: String,
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    pub volume: Quantity,
    pub start_ns: u64,
    pub end_ns: u64,
}

impl DeterministicState for MarketTick {
    fn state_hash(&self) -> [u8; 32] {
        hash_bytes(&serialize_canonical(self).expect("MarketTick serialization failed"))
    }
}

impl DeterministicState for OHLCVBar {
    fn state_hash(&self) -> [u8; 32] {
        hash_bytes(&serialize_canonical(self).expect("OHLCVBar serialization failed"))
    }
}
