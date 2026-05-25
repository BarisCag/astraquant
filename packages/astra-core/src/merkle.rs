use crate::hashing::{hash_bytes, DeterministicState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MerkleNode {
    Leaf {
        hash: [u8; 32],
    },
    Node {
        left: Box<MerkleNode>,
        right: Box<MerkleNode>,
        hash: [u8; 32],
    },
}

impl MerkleNode {
    pub fn hash(&self) -> &[u8; 32] {
        match self {
            Self::Leaf { hash } => hash,
            Self::Node { hash, .. } => hash,
        }
    }
}

pub struct MerkleTree {
    pub root: Option<MerkleNode>,
}

impl MerkleTree {
    pub fn build(leaves: &[[u8; 32]]) -> Self {
        if leaves.is_empty() {
            return Self { root: None };
        }
        let nodes: Vec<MerkleNode> = leaves
            .iter()
            .map(|h| MerkleNode::Leaf { hash: *h })
            .collect();
        let root = Self::build_layer(nodes);
        Self {
            root: Some(root[0].clone()),
        }
    }

    fn build_layer(nodes: Vec<MerkleNode>) -> Vec<MerkleNode> {
        if nodes.len() == 1 {
            return nodes;
        }
        let mut next = Vec::new();
        for chunk in nodes.chunks(2) {
            if chunk.len() == 2 {
                let mut combined = Vec::new();
                combined.extend_from_slice(chunk[0].hash());
                combined.extend_from_slice(chunk[1].hash());
                next.push(MerkleNode::Node {
                    left: Box::new(chunk[0].clone()),
                    right: Box::new(chunk[1].clone()),
                    hash: hash_bytes(&combined),
                });
            } else {
                next.push(chunk[0].clone());
            }
        }
        Self::build_layer(next)
    }

    pub fn root_hash(&self) -> Option<[u8; 32]> {
        self.root.as_ref().map(|r| *r.hash())
    }
}

impl DeterministicState for MerkleTree {
    fn state_hash(&self) -> [u8; 32] {
        self.root_hash().unwrap_or([0; 32])
    }
}
