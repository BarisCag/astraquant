use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CHF,
    TRY, // Essential for local corporate modeling (e.g., BOSSA operations)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tenor {
    Spot,        // T+0 to T+2
    OneWeek,     // T+3 to T+7
    OneMonth,    // T+8 to T+30
    ThreeMonths, // T+31 to T+90
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashPosition {
    pub currency: Currency,
    pub available_balance: Decimal,
    pub pending_inflows: Decimal,
    pub pending_outflows: Decimal,
    pub accrued_interest: Decimal,
    /// Calendar days until the next major settlement clears
    pub next_settlement_days: u32,
    /// Deterministic liquidity runway (Cash Conversion Cycle proxy)
    pub liquidity_runway_days: u32, 
}

impl CashPosition {
    /// Calculates net available liquidity assuming all pending flows settle
    pub fn net_projected_balance(&self) -> Decimal {
        self.available_balance + self.pending_inflows - self.pending_outflows
    }
}
