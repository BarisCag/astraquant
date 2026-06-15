use bincode::Options;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ExchangeStateHash {
    pub risk_engine_hash: [u8; 32],
    pub portfolio_engine_hash: [u8; 32],
    pub matching_engines_hash: [u8; 32],
    pub diagnostics_hash: [u8; 32],
    pub strategy_runtime_hash: [u8; 32],
    pub settlement_engine_hash: [u8; 32],
    pub margin_engine_hash: [u8; 32],
    pub funding_ledger_hash: [u8; 32],
    pub sequence_clock: u64,
}

impl ExchangeStateHash {
    pub fn state_hash(&self) -> [u8; 32] {
        let bytes = bincode::options()
            .with_little_endian()
            .with_fixint_encoding()
            .serialize(self)
            .expect("ExchangeStateHash serialization failed");
        let mut hasher = blake3::Hasher::new();
        hasher.update(&bytes);
        *hasher.finalize().as_bytes()
    }
}
