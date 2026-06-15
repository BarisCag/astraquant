use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettlementFailure {
    pub trader_id: u64,
    pub symbol: String,
    pub sequence_id: u64,
    pub failed_amount: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CounterpartyDefault {
    pub trader_id: u64,
    pub sequence_id: u64,
    pub deficit: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiquidationCascade {
    pub source_trader_id: u64,
    pub sequence_id: u64,
    pub cascade_depth: u64,
}
