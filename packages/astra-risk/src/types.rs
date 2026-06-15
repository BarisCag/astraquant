use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraderRiskProfile {
    pub trader_id: u64,
    pub max_position_notional: i64,
    pub max_order_quantity: u64,
    pub max_drawdown: i64,
    pub max_order_velocity: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskViolation {
    MaxPositionExceeded,
    MaxOrderQuantityExceeded,
    MaxDrawdownExceeded,
    MaxOrderVelocityExceeded,
    InvalidOrder,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TraderExposure {
    pub trader_id: u64,
    pub gross_exposure: i64,
    pub net_exposure: i64,
    pub realized_pnl: i64,
    pub open_orders: u64,
}
