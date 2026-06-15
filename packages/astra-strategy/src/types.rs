use astra_core::types::{Price, Quantity};
use astra_lob::types::OrderSide;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum StrategyAction {
    SubmitOrder {
        symbol: String,
        side: OrderSide,
        price: Price,
        quantity: Quantity,
    },
    CancelOrder {
        symbol: String,
        order_id: u64,
    },
    ModifyOrder {
        symbol: String,
        order_id: u64,
        new_price: Price,
        new_quantity: Quantity,
    },
    PauseTrading {
        duration_sequences: u64,
    },
    ReduceInventory {
        target_quantity: Quantity,
    },
}

#[derive(Clone, Debug)]
pub enum MarketEvent {
    BookUpdate {
        engine_sequence_id: u64,
        symbol: String,
        best_bid: Option<Price>,
        best_ask: Option<Price>,
        bid_depth: Quantity,
        ask_depth: Quantity,
    },
    TradeExecution {
        engine_sequence_id: u64,
        symbol: String,
        price: Price,
        quantity: Quantity,
        is_buyer_maker: bool,
    },
}
