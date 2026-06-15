use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CircuitBreakerRule {
    pub volatility_threshold_ppm: u64,
    pub halt_duration_sequences: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ShortSaleRestriction {
    pub volatility_threshold_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VolatilityInterruption {
    pub volatility_threshold_ppm: u64,
    pub auction_duration_sequences: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VenueParticipationRule {
    pub fragmentation_threshold_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SettlementFreezePolicy {
    pub insolvency_threshold_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CollateralEscalationRule {
    pub funding_failure_threshold_ppm: u64,
}
