use astra_lob::diagnostics::ReplayDiagnostics;
use bincode::Options;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ExchangeDiagnostics {
    pub total_events_processed: u64,
    pub total_accepted_orders: u64,
    pub total_rejected_orders: u64,
    pub lob_diagnostics: ReplayDiagnostics,
}

impl ExchangeDiagnostics {
    pub fn state_hash(&self) -> [u8; 32] {
        let bytes = bincode::options()
            .with_little_endian()
            .with_fixint_encoding()
            .serialize(self)
            .expect("ExchangeDiagnostics serialization failed");
        let mut hasher = blake3::Hasher::new();
        hasher.update(&bytes);
        *hasher.finalize().as_bytes()
    }
}
