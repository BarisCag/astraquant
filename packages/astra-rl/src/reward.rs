use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RewardFunction {
    Pnl,
    RiskAdjusted,
}

impl RewardFunction {
    pub fn evaluate(&self, pnl: i64, risk_penalty: i64) -> f64 {
        match self {
            RewardFunction::Pnl => pnl as f64,
            RewardFunction::RiskAdjusted => (pnl - risk_penalty) as f64,
        }
    }
}
