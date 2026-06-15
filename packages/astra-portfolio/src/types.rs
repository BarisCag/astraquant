use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    pub trader_id: u64,
    pub symbol: String,
    pub net_quantity: i64,
    pub average_entry_price: i64,
    pub realized_pnl: i64,
    pub unrealized_pnl: i64,
    pub last_mark_price: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortfolioSnapshot {
    pub gross_exposure: i64,
    pub net_exposure: i64,
    pub total_realized_pnl: i64,
    pub total_unrealized_pnl: i64,
    pub active_symbol_count: u64,
    pub inventory_concentration_metrics: BTreeMap<String, i64>,
}
