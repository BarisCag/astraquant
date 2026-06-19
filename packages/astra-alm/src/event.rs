use serde::{Deserialize, Serialize};
use crate::types::{ALMMismatchReport, Currency, TenorBucket};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HedgeRecommendation {
    pub instrument: String,
    pub target_notional: i64,
    pub hedge_ratio: f64,
    pub confidence: f64,
    pub policy_version_hash: [u8; 32],
    pub rationale: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ALMEvent {
    HedgeRecommendation(HedgeRecommendation),
    RebalancingRecommended {
        currency: Currency,
        tenor: TenorBucket,
        duration_gap: f64,
        recommended_action: String,
    },
    ALMMismatchReport(Vec<ALMMismatchReport>),
}
