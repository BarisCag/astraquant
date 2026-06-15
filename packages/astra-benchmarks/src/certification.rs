use serde::{Deserialize, Serialize};
use astra_core::hashing::hash_bytes;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineageCheckpoint {
    pub sequence_id: u64,
    pub global_state_hash: [u8; 32],
    pub journal_window_hash: [u8; 32],
    pub policy_state_hash: [u8; 32],
    pub clearing_state_hash: [u8; 32],
    pub scenario_state_hash: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificationWindow {
    pub start_sequence: u64,
    pub end_sequence: u64,
    pub checkpoints: Vec<LineageCheckpoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayLineageTree {
    pub root_hash: [u8; 32],
    pub windows: Vec<CertificationWindow>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayParityProof {
    pub lineage_tree: ReplayLineageTree,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkTerminalCertificate {
    pub run_id: String,
    pub final_sequence: u64,
    pub terminal_hash: [u8; 32],
    pub parity_proof: ReplayParityProof,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayCertificationManifest {
    pub manifest_id: String,
    pub benchmark_id: String,
    pub base_seed: u64,
    pub certificate: BenchmarkTerminalCertificate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HashParityCertificate {
    pub certified_hash: [u8; 32],
    pub verifier: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayIntegrityReport {
    pub match_status: bool,
    pub divergent_sequence: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineageVerificationReport {
    pub valid: bool,
    pub corrupted_window: Option<u64>,
}

// Phase 13C: Deterministic Audit Extensions

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvariantVerificationWindow {
    pub start_sequence: u64,
    pub end_sequence: u64,
    pub invariants_checked: u64,
    pub violations_detected: u64,
    pub compliance_score_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineageProofChain {
    pub chain_id: String,
    pub proofs: Vec<LineageCheckpoint>,
    pub terminal_hash: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeterministicAuditSignature {
    pub signer_id: String,
    pub signed_hash: [u8; 32],
    pub sequence_at_signing: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkIntegrityAssertion {
    pub benchmark_id: String,
    pub invariant_window: InvariantVerificationWindow,
    pub lineage_chain: LineageProofChain,
    pub audit_signature: DeterministicAuditSignature,
    pub certification_valid: bool,
}
