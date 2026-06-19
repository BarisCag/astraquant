use crate::event::HedgeRecommendation;

// TODO: Replace with ort ONNX inference in future phase
pub struct DRLBridge;

impl DRLBridge {
    pub fn new() -> Self {
        Self
    }

    pub fn recommend(
        &self,
        _exposure_vector: &[f64],
        _prev_hash: [u8; 32],
    ) -> HedgeRecommendation {
        let policy_version_hash = *blake3::hash(b"stub_policy_v1").as_bytes();

        HedgeRecommendation {
            instrument: "STUB_HEDGE_INSTRUMENT".to_string(),
            target_notional: 0,
            hedge_ratio: 0.5,
            confidence: 0.75,
            policy_version_hash,
            rationale: "STUB: real DRL policy not loaded".to_string(),
        }
    }
}

impl Default for DRLBridge {
    fn default() -> Self {
        Self::new()
    }
}
