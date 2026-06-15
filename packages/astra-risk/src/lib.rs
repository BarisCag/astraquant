pub mod engine;
pub mod types;
pub mod velocity;
pub mod exposure;
pub mod margin;
pub mod stress;
pub mod limits;
pub mod certification;

/// Inherent deterministic hashing for RiskEngine.
impl engine::RiskEngine {
    pub fn state_hash(&self) -> [u8; 32] {
        let bytes = bincode::serialize(self).expect("RiskEngine canonical serialization failed");
        let hash = blake3::hash(&bytes);
        *hash.as_bytes()
    }
}

impl types::TraderExposure {
    pub fn state_hash(&self) -> [u8; 32] {
        let bytes =
            bincode::serialize(self).expect("TraderExposure canonical serialization failed");
        let hash = blake3::hash(&bytes);
        *hash.as_bytes()
    }
}

impl velocity::VelocityWindow {
    pub fn state_hash(&self) -> [u8; 32] {
        let bytes =
            bincode::serialize(self).expect("VelocityWindow canonical serialization failed");
        let hash = blake3::hash(&bytes);
        *hash.as_bytes()
    }
}
