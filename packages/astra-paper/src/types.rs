use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderType {
    Market,
    Limit(u64),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaperOrder {
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaperFill {
    pub symbol: String,
    pub side: Side,
    pub fill_price: u64,
    pub fill_quantity: u64,
    pub timestamp_ns: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketSnapshot {
    pub symbol: String,
    pub last_price: u64,
    pub timestamp_ns: u64,
}
