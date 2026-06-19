use serde::{Deserialize, Serialize};
use crate::types::{MarketSnapshot, PaperOrder, Side};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FillEvent {
    pub symbol: String,
    pub side: Side,
    pub fill_price: u64,
    pub fill_quantity: u64,
    pub timestamp_ns: u64,
    pub model_used: String,
    pub prng_nonce_at_fill: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskAlert {
    pub timestamp_ns: u64,
    pub reason: String,
    pub halted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    pub timestamp_ns: u64,
    pub cash_balance: i64,
    pub realized_pnl: i64,
    pub unrealized_pnl: i64,
    pub total_nav: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PaperEvent {
    MarketData(MarketSnapshot),
    StrategySignal(Vec<PaperOrder>),
    Fill(FillEvent),
    RiskBreach(RiskAlert),
    PortfolioSnapshot(PortfolioSnapshot),
}
