use crate::events::{AstraEvent, EventType};
use crate::hashing::{hash_bytes, DeterministicState};
use crate::replay::EventReducer;
use crate::serialization::{deserialize_canonical, serialize_canonical};
use crate::types::{Price, Quantity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderSide {
    Bid,
    Ask,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LimitOrderPlacedPayload {
    pub order_id: u64,
    pub trader_id: u64,
    pub symbol: String,
    pub side: OrderSide,
    pub price: Price,
    pub quantity: Quantity,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LimitOrderCancelledPayload {
    pub order_id: u64,
    pub symbol: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LimitOrderMatchedPayload {
    pub maker_order_id: u64,
    pub taker_order_id: u64,
    pub maker_trader_id: u64,
    pub taker_trader_id: u64,
    pub match_price: Price,
    pub matched_quantity: Quantity,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderNode {
    pub order_id: u64,
    pub trader_id: u64,
    pub quantity: Quantity,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: BTreeMap<Price, Vec<OrderNode>>,
    pub asks: BTreeMap<Price, Vec<OrderNode>>,
    pub order_index: BTreeMap<u64, (OrderSide, Price)>,
    pub last_applied_sequence_id: Option<u64>,
}

#[derive(Debug)]
pub enum OrderBookError {
    DeserializationFailed(String),
    OrderAlreadyExists(u64),
    OrderNotFound(u64),
    InsufficientQuantity(u64),
}

impl fmt::Display for OrderBookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeserializationFailed(e) => write!(f, "Deserialization failed: {}", e),
            Self::OrderAlreadyExists(id) => write!(f, "Order {} already exists", id),
            Self::OrderNotFound(id) => write!(f, "Order {} not found", id),
            Self::InsufficientQuantity(id) => write!(f, "Order {} has insufficient quantity", id),
        }
    }
}

impl OrderBook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_index: BTreeMap::new(),
            last_applied_sequence_id: None,
        }
    }

    pub fn apply_placed(
        &mut self,
        payload: &LimitOrderPlacedPayload,
    ) -> Result<(), OrderBookError> {
        if self.symbol != payload.symbol {
            return Ok(());
        }
        if self.order_index.contains_key(&payload.order_id) {
            return Err(OrderBookError::OrderAlreadyExists(payload.order_id));
        }

        let node = OrderNode {
            order_id: payload.order_id,
            trader_id: payload.trader_id,
            quantity: payload.quantity,
        };

        match payload.side {
            OrderSide::Bid => {
                self.bids.entry(payload.price).or_default().push(node);
            }
            OrderSide::Ask => {
                self.asks.entry(payload.price).or_default().push(node);
            }
        }
        self.order_index
            .insert(payload.order_id, (payload.side, payload.price));

        Ok(())
    }

    pub fn apply_cancelled(
        &mut self,
        payload: &LimitOrderCancelledPayload,
    ) -> Result<(), OrderBookError> {
        if self.symbol != payload.symbol {
            return Ok(());
        }

        let (side, price) = match self.order_index.remove(&payload.order_id) {
            Some(entry) => entry,
            None => return Err(OrderBookError::OrderNotFound(payload.order_id)),
        };

        let level = match side {
            OrderSide::Bid => self.bids.get_mut(&price),
            OrderSide::Ask => self.asks.get_mut(&price),
        };

        if let Some(nodes) = level {
            nodes.retain(|n| n.order_id != payload.order_id);
            if nodes.is_empty() {
                match side {
                    OrderSide::Bid => self.bids.remove(&price),
                    OrderSide::Ask => self.asks.remove(&price),
                };
            }
        }

        Ok(())
    }

    pub fn apply_matched(
        &mut self,
        payload: &LimitOrderMatchedPayload,
    ) -> Result<(), OrderBookError> {
        let (side, price) = match self.order_index.get(&payload.maker_order_id) {
            Some(&(s, p)) => (s, p),
            None => return Ok(()),
        };

        let level = match side {
            OrderSide::Bid => self.bids.get_mut(&price),
            OrderSide::Ask => self.asks.get_mut(&price),
        };

        if let Some(nodes) = level {
            if let Some(node) = nodes
                .iter_mut()
                .find(|n| n.order_id == payload.maker_order_id)
            {
                if let Some(rem) = node.quantity.checked_sub(payload.matched_quantity) {
                    node.quantity = rem;
                    if rem.0 == 0 {
                        nodes.retain(|n| n.order_id != payload.maker_order_id);
                        self.order_index.remove(&payload.maker_order_id);
                    }
                } else {
                    return Err(OrderBookError::InsufficientQuantity(payload.maker_order_id));
                }
            }
            if nodes.is_empty() {
                match side {
                    OrderSide::Bid => self.bids.remove(&price),
                    OrderSide::Ask => self.asks.remove(&price),
                };
            }
        }

        Ok(())
    }
}

impl DeterministicState for OrderBook {
    fn state_hash(&self) -> [u8; 32] {
        let bytes = serialize_canonical(self).expect("OrderBook canonical serialization failed");
        hash_bytes(&bytes)
    }
}

impl EventReducer for OrderBook {
    type Error = OrderBookError;

    fn apply(&mut self, event: &AstraEvent) -> Result<(), Self::Error> {
        match event.event_type {
            EventType::LimitOrderPlaced => {
                let payload: LimitOrderPlacedPayload = deserialize_canonical(&event.payload)
                    .map_err(|e| OrderBookError::DeserializationFailed(e.to_string()))?;
                self.apply_placed(&payload)?;
            }
            EventType::LimitOrderCancelled => {
                let payload: LimitOrderCancelledPayload = deserialize_canonical(&event.payload)
                    .map_err(|e| OrderBookError::DeserializationFailed(e.to_string()))?;
                self.apply_cancelled(&payload)?;
            }
            EventType::LimitOrderMatched => {
                let payload: LimitOrderMatchedPayload = deserialize_canonical(&event.payload)
                    .map_err(|e| OrderBookError::DeserializationFailed(e.to_string()))?;
                self.apply_matched(&payload)?;
            }
            _ => {}
        }

        self.last_applied_sequence_id = Some(event.sequence_id);
        Ok(())
    }

    fn last_applied_sequence_id(&self) -> Option<u64> {
        self.last_applied_sequence_id
    }
}
