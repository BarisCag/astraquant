use crate::certification::BenchmarkTerminalCertificate;
use astra_scenarios::scenario::ScenarioDefinition;
use serde::{Deserialize, Serialize};

pub trait BenchmarkDefinition {
    fn benchmark_id(&self) -> &'static str;
    fn get_version(&self) -> u32;
    fn get_scenario(&self) -> Box<dyn ScenarioDefinition>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkScenario {
    pub id: String,
    pub seed: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkRun {
    pub run_id: String,
    pub benchmark_id: String,
    pub scenario: BenchmarkScenario,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkManifest {
    pub benchmark_version: u32,
    pub scenario_revision: u32,
    pub replay_protocol_version: u32,
    pub policy_model_revision: u32,
    pub certification_schema_version: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkCertification {
    pub run_id: String,
    pub manifest: BenchmarkManifest,
    pub terminal_certificate: BenchmarkTerminalCertificate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkReplayPack {
    pub certification: BenchmarkCertification,
    // Typically contains encoded diffs or full event streams
    pub trace_blob_hash: [u8; 32],
}
