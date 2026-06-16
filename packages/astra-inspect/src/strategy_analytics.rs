use astra_strategy::agent::AgentContext;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StrategyAnalytics {
    pub inventory_series: BTreeMap<u64, i64>,
    pub realized_pnl_series: BTreeMap<u64, i64>,
    pub unrealized_pnl_series: BTreeMap<u64, i64>,
    pub average_quote_lifetime_ns: u64,
    pub total_fills: u64,
    pub total_cancellations: u64,
}

pub struct StrategyAnalyticsCollector {
    pub metrics_by_trader: BTreeMap<u64, StrategyAnalytics>,
}

impl Default for StrategyAnalyticsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategyAnalyticsCollector {
    pub fn new() -> Self {
        Self {
            metrics_by_trader: BTreeMap::new(),
        }
    }

    pub fn record_context(&mut self, timestamp_ns: u64, trader_id: u64, ctx: &AgentContext) {
        let metrics = self.metrics_by_trader.entry(trader_id).or_default();
        metrics.inventory_series.insert(timestamp_ns, ctx.inventory);
        metrics
            .realized_pnl_series
            .insert(timestamp_ns, ctx.realized_pnl);
        metrics
            .unrealized_pnl_series
            .insert(timestamp_ns, ctx.unrealized_pnl);
    }
}
