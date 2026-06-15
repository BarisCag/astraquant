use crate::types::{Money, Quantity};
pub use astra_risk::engine::RiskEngine;
pub use astra_risk::types::{RiskViolation, TraderExposure, TraderRiskProfile};

pub fn create_default_risk_engine(max_notional: Money, max_qty: Quantity) -> RiskEngine {
    let mut engine = RiskEngine::new();
    engine.register_trader(TraderRiskProfile {
        trader_id: 1, // Default trader
        max_position_notional: max_notional.0 as i64,
        max_order_quantity: max_qty.0,
        max_drawdown: max_notional.0 as i64,
        max_order_velocity: 100_000,
    });
    engine
}
