use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PolicyAction {
    CircuitBreaker { symbol: String },
    ShortSellingBan { symbol: String },
    EmergencyCollateralEasing { haircut_ppm: u64 },
    LiquidityInjection { amount: u64 },
    VolatilityInterruption { symbol: String },
    FundingFacility { total_capacity: u64 },
    SettlementHoliday { duration_sequences: u64 },
    RepoFacility { target_rate_ppm: u64 },
    CapitalControls { withdrawal_limit: u64 },
    VenueConstraint { venue_id: u32 },
    MarketMakerObligation { symbol: String, spread_ppm: u64 },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PolicySeverityTier {
    Level1,
    Level2,
    Level3,
    Emergency,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PolicyExecutionWindow {
    pub start_sequence: u64,
    pub end_sequence: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PolicyDirective {
    pub directive_id: u64,
    pub action: PolicyAction,
    pub severity: PolicySeverityTier,
    pub execution_window: PolicyExecutionWindow,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PolicyCheckpoint {
    pub checkpoint_sequence: u64,
    pub active_directives: Vec<PolicyDirective>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PolicyOutcome {
    pub directive_id: u64,
    pub stabilization_score_ppm: u64,
}
