use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayLineageNode {
    pub sequence_id: u64,
    pub state_hash: [u8; 32],
    pub parent_hash: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineageGraph {
    pub nodes: Vec<ReplayLineageNode>,
    pub root_hash: [u8; 32],
}

impl LineageGraph {
    pub fn new(root_hash: [u8; 32]) -> Self {
        Self {
            nodes: Vec::new(),
            root_hash,
        }
    }

    pub fn add_node(&mut self, node: ReplayLineageNode) {
        self.nodes.push(node);
    }

    pub fn verify_continuity(&self) -> bool {
        if self.nodes.is_empty() {
            return true;
        }
        // First node's parent_hash must match root_hash
        if self.nodes[0].parent_hash != self.root_hash {
            return false;
        }
        // Each subsequent node's parent_hash must match previous node's state_hash
        for i in 1..self.nodes.len() {
            if self.nodes[i].parent_hash != self.nodes[i - 1].state_hash {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointLineageProof {
    pub checkpoint_interval: u64,
    pub proofs: Vec<ReplayLineageNode>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificationChain {
    pub chain_id: String,
    pub lineage: LineageGraph,
    pub terminal_hash: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HashContinuityWindow {
    pub start_sequence: u64,
    pub end_sequence: u64,
    pub entry_hash: [u8; 32],
    pub exit_hash: [u8; 32],
}
