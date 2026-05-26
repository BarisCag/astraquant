use astra_core::types::{Price, Quantity};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BookSnapshot {
    pub symbol: String,
    pub best_bid: Option<Price>,
    pub best_ask: Option<Price>,
    pub spread_ticks: i64,
    pub total_bid_levels: u64,
    pub total_ask_levels: u64,
    pub total_bid_liquidity: Quantity,
    pub total_ask_liquidity: Quantity,
}
