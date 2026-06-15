use crate::scenario::ScenarioRuntime;
use astra_exchange::state::ExchangeStateHash;
use bincode::Options;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioCheckpoint {
    pub scenario_runtime: ScenarioRuntime,
    pub exchange_hash: ExchangeStateHash,
    pub checkpoint_sequence: u64,
    pub integrity_hash: [u8; 32],
}

impl ScenarioCheckpoint {
    pub fn new(scenario_runtime: ScenarioRuntime, exchange_hash: ExchangeStateHash) -> Self {
        let mut chk = Self {
            checkpoint_sequence: scenario_runtime.current_sequence,
            scenario_runtime,
            exchange_hash,
            integrity_hash: [0; 32],
        };
        chk.integrity_hash = chk.compute_hash();
        chk
    }

    pub fn compute_hash(&self) -> [u8; 32] {
        let bytes = bincode::options()
            .with_little_endian()
            .with_fixint_encoding()
            .serialize(&(
                &self.scenario_runtime,
                &self.exchange_hash.state_hash(),
                &self.checkpoint_sequence,
            ))
            .expect("Serialization failed");

        let mut hasher = blake3::Hasher::new();
        hasher.update(&bytes);
        *hasher.finalize().as_bytes()
    }
}
