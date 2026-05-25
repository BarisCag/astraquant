use crate::events::{AstraEvent, EventType, PayloadMetadata};
use crate::hashing::DeterministicState;
use crate::orderbook::OrderSide;
use crate::types::{Price, Quantity};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum StrategyAction {
    SubmitLimitOrder {
        symbol: String,
        side: OrderSide,
        price: Price,
        quantity: Quantity,
    },
    CancelOrder {
        order_id: u64,
    },
    EmitEvent {
        event_type: EventType,
        payload: Vec<u8>,
        metadata: PayloadMetadata,
    },
}

pub trait Strategy: DeterministicState {
    fn strategy_id(&self) -> u64;

    fn on_event(&mut self, event: &AstraEvent) -> Result<Vec<StrategyAction>, String>;
}
