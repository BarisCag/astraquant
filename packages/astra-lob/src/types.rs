use astra_core::types::{Price, Quantity};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderSide {
    Bid,
    Ask,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Order {
    pub order_id: u64,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Price,
    pub quantity: Quantity,
    pub remaining_quantity: Quantity,
    pub timestamp_ns: u64,
    pub trader_id: u64,
    pub queue_position: crate::queue::QueuePosition,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LiquiditySide {
    Maker,
    Taker,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TradeExecution {
    pub resting_order_id: u64,
    pub aggressive_order_id: u64,
    pub symbol: String,
    pub match_price: Price,
    pub matched_quantity: Quantity,
    pub liquidity_side: LiquiditySide,
    pub timestamp_ns: u64,
    pub trader_id: u64,
    pub queue_position: crate::queue::QueuePosition,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderEvent {
    Accepted(Order),
    Rejected {
        order_id: u64,
        reason: String,
    },
    TradeExecuted(TradeExecution),
    Cancelled {
        order_id: u64,
        symbol: String,
        reason: String,
    },
    Modified {
        order_id: u64,
        symbol: String,
        new_quantity: Quantity,
    },
    DestinationUnavailable {
        order_id: u64,
        venue_id: u8,
        reason: String,
    },
}
