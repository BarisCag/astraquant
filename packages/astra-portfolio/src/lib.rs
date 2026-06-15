pub mod engine;
pub mod types;

use bincode::Options;

impl engine::PositionEngine {
    pub fn state_hash(&self) -> [u8; 32] {
        let bytes = bincode::options()
            .with_little_endian()
            .with_fixint_encoding()
            .serialize(self)
            .expect("PositionEngine serialization failed");
        let mut hasher = blake3::Hasher::new();
        hasher.update(&bytes);
        *hasher.finalize().as_bytes()
    }
}

impl types::PortfolioSnapshot {
    pub fn state_hash(&self) -> [u8; 32] {
        let bytes = bincode::options()
            .with_little_endian()
            .with_fixint_encoding()
            .serialize(self)
            .expect("PortfolioSnapshot serialization failed");
        let mut hasher = blake3::Hasher::new();
        hasher.update(&bytes);
        *hasher.finalize().as_bytes()
    }
}
