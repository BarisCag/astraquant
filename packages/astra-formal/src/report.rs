use serde::{Deserialize, Serialize};
use crate::invariant::InvariantProof;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormalVerificationReport {
    pub report_id: String,
    pub is_valid: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeterminismCertificationReport {
    pub certification_id: String,
    pub terminal_hash: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayEquivalenceReport {
    pub is_equivalent: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DistributedParityCertification {
    pub cluster_id: String,
    pub parity_matched: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvariantViolationManifest {
    pub violations: Vec<InvariantProof>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificationTrustChain {
    pub chain_id: String,
    pub trust_score: u64,
}

// Phase 17A: Federation Formal Reports

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FederatedDeterminismCertification {
    pub federation_id: String,
    pub global_parity_hash: [u8; 32],
    pub is_formally_verified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrossClusterEquivalenceReport {
    pub source_cluster_id: String,
    pub target_cluster_id: String,
    pub equivalence_matched: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayTreatyVerificationReport {
    pub treaty_id: String,
    pub invariant_violations: u64,
    pub is_compliant: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FederationIntegrityManifest {
    pub manifest_id: String,
    pub total_clusters_verified: u64,
    pub integrity_score_ppm: u64,
}
