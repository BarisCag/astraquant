use crate::hashing::hash_bytes;
use crate::serialization::serialize_canonical;
use crate::types::{Price, Quantity};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TradeId(pub u64);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Trade {
    pub trade_id: TradeId,
    pub symbol: String,
    pub taker_side: crate::orderbook::OrderSide,
    pub maker_order_id: u64,
    pub taker_order_id: u64,
    pub price: Price,
    pub quantity: Quantity,
    pub timestamp_ns: u64,
}

impl Trade {
    pub fn trade_hash(&self) -> [u8; 32] {
        let bytes = serialize_canonical(self).expect("Trade canonical serialization failed");
        hash_bytes(&bytes)
    }
}
