use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SequenceProofWindow {
    pub start_sequence: u64,
    pub end_sequence: u64,
    pub proof_hash: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayProof {
    pub proof_id: String,
    pub window: SequenceProofWindow,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayProofChain {
    pub chain_id: String,
    pub proofs: Vec<ReplayProof>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanonicalStateProof {
    pub state_hash: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayEquivalenceProof {
    pub expected_hash: [u8; 32],
    pub actual_hash: [u8; 32],
    pub is_equivalent: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofCertificationManifest {
    pub manifest_id: String,
    pub terminal_proof_hash: [u8; 32],
}
