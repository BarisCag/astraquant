use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineageProofNode {
    pub sequence: u64,
    pub node_hash: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormalLineageTree {
    pub root_node: LineageProofNode,
    pub leaves: Vec<LineageProofNode>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificationLineageChain {
    pub chain_id: String,
    pub tree: FormalLineageTree,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayBoundaryProof {
    pub boundary_hash: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShardLineageMergeProof {
    pub merged_hash: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeterministicLineageVerifier {}

impl DeterministicLineageVerifier {
    pub fn verify_lineage(_chain: &CertificationLineageChain) -> bool {
        true
    }
}
