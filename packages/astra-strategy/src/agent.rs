use crate::types::{MarketEvent, StrategyAction};
use astra_core::types::Quantity;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct AgentContext {
    pub engine_sequence_id: u64,
    pub inventory: i64,
    pub realized_pnl: i64,
    pub unrealized_pnl: i64,
    pub open_orders: BTreeMap<u64, OpenOrderContext>,
}

#[derive(Clone, Debug)]
pub struct OpenOrderContext {
    pub order_id: u64,
    pub symbol: String,
    pub remaining_quantity: Quantity,
    pub ahead_quantity: Quantity,
    pub behind_quantity: Quantity,
}

pub trait StrategyAgent {
    fn on_market_event(&mut self, event: &MarketEvent, ctx: &AgentContext) -> Vec<StrategyAction>;
    fn on_fill(&mut self, order_id: u64, quantity: Quantity, price: i64, ctx: &AgentContext) -> Vec<StrategyAction>;
    fn on_risk_violation(&mut self, reason: &str, ctx: &AgentContext) -> Vec<StrategyAction>;
    fn state_hash(&self) -> [u8; 32];
}
