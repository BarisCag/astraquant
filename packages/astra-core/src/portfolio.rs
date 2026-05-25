use crate::events::AstraEvent;
use crate::hashing::{hash_bytes, DeterministicState};
use crate::orderbook::OrderSide;
use crate::replay::EventReducer;
use crate::serialization::{deserialize_canonical, serialize_canonical};
use crate::types::{Money, Price, Quantity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    pub symbol: String,
    pub quantity: i64,
    pub average_entry_price: Price,
    pub realized_pnl: Money,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Portfolio {
    pub positions: BTreeMap<String, Position>,
    pub cash_balance: Money,
    pub last_applied_sequence_id: Option<u64>,
}

#[derive(Debug)]
pub enum PortfolioError {
    DeserializationFailed(String),
}

impl fmt::Display for PortfolioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeserializationFailed(e) => write!(f, "Portfolio deserialization failed: {}", e),
        }
    }
}

impl Default for Portfolio {
    fn default() -> Self {
        Self::new()
    }
}

impl Portfolio {
    pub fn new() -> Self {
        Self {
            positions: BTreeMap::new(),
            cash_balance: Money::new(0),
            last_applied_sequence_id: None,
        }
    }

    pub fn apply_trade_settled(
        &mut self,
        symbol: &str,
        side: OrderSide,
        price: Price,
        qty: Quantity,
    ) {
        let pos = self
            .positions
            .entry(symbol.to_string())
            .or_insert(Position {
                symbol: symbol.to_string(),
                quantity: 0,
                average_entry_price: Price::new(0),
                realized_pnl: Money::new(0),
            });

        let qty_i64 = qty.0 as i64;
        let price_i128 = price.0 as i128;

        match side {
            OrderSide::Bid => {
                pos.quantity = pos.quantity.checked_add(qty_i64).unwrap_or(pos.quantity);
                self.cash_balance.0 = self
                    .cash_balance
                    .0
                    .checked_sub(price_i128 * (qty_i64 as i128))
                    .unwrap_or(self.cash_balance.0);
            }
            OrderSide::Ask => {
                pos.quantity = pos.quantity.checked_sub(qty_i64).unwrap_or(pos.quantity);
                self.cash_balance.0 = self
                    .cash_balance
                    .0
                    .checked_add(price_i128 * (qty_i64 as i128))
                    .unwrap_or(self.cash_balance.0);
            }
        }
    }
}

impl DeterministicState for Portfolio {
    fn state_hash(&self) -> [u8; 32] {
        let bytes = serialize_canonical(self).expect("Portfolio serialization failed");
        hash_bytes(&bytes)
    }
}

impl EventReducer for Portfolio {
    type Error = PortfolioError;

    fn apply(&mut self, event: &AstraEvent) -> Result<(), Self::Error> {
        if event.event_type == crate::events::EventType::TradeSettled {
            if let Ok(trade) = deserialize_canonical::<crate::trades::Trade>(&event.payload) {
                self.apply_trade_settled(
                    &trade.symbol,
                    trade.taker_side,
                    trade.price,
                    trade.quantity,
                );
            }
        }
        self.last_applied_sequence_id = Some(event.sequence_id);
        Ok(())
    }

    fn last_applied_sequence_id(&self) -> Option<u64> {
        self.last_applied_sequence_id
    }
}
