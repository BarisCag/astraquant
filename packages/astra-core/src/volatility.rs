use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityTick {
    pub asset: String,
    pub implied_volatility: f64,
}
