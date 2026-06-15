use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct SystemicPropagationMetrics {
    pub liquidity_recovery_score_ppm: u64,
    pub contagion_dampening_ppm: u64,
    pub stabilization_effectiveness_ppm: u64,
    pub insolvency_containment_ppm: u64,
    pub venue_normalization_score_ppm: u64,
}

impl SystemicPropagationMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}
