use crate::types::{CashPosition, Currency, Tenor};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TreasuryEvent {
    /// Emitted when cash balances or pending settlements change
    CashPositionUpdate {
        position: CashPosition,
    },
    
    /// Emitted by the CashFlowEngine (Deterministic 30-day projection)
    CashFlowForecast {
        currency: Currency,
        projected_inflows: Decimal,
        projected_outflows: Decimal,
        confidence_score: Decimal, // Always 1.0 for scheduled, < 1.0 for heuristic projections
    },

    /// Emitted by the FxExposureTracker for downstream ALM/Risk consumption
    FxExposureSnapshot {
        base: Currency,
        quote: Currency,
        tenor: Tenor,
        net_delta: Decimal,
    },

    /// Soft signal emitted when stress tests fail (e.g., 3-day zero inflow LCR test)
    LiquidityWarning {
        currency: Currency,
        shortfall_amount: Decimal,
        metric_breached: String, // e.g., "LCR_3_DAY_STRESS"
    },

    /// Daily interest accrual updates independent of trading activity
    AccrualUpdate {
        currency: Currency,
        accrued_amount: Decimal,
    },
}
