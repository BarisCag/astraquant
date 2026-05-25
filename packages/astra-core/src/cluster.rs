use crate::consensus::ConsensusState;
use crate::hashing::{hash_bytes, DeterministicState};
use crate::kernel::AstraKernel;
use crate::replay::EventReducer;
use crate::replication::ReplicationBuffer;
use crate::sync::Synchronizer;
use crate::transport::TransportPacket;
use crate::verification::VerificationManifest;

pub struct ClusterNode {
    pub node_id: u64,
    pub kernel: AstraKernel,
    pub consensus: ConsensusState,
    pub replication: ReplicationBuffer,
}

impl ClusterNode {
    pub fn new(node_id: u64, kernel: AstraKernel) -> Self {
        Self {
            node_id,
            kernel,
            consensus: ConsensusState::new(),
            replication: ReplicationBuffer::new(),
        }
    }

    pub fn receive_packet(&mut self, packet: &TransportPacket) -> Result<(), String> {
        if let TransportPacket::AppendEntries { entries, .. } = packet {
            for event in entries {
                self.replication.push(event.clone());
                Synchronizer::apply_replicated_events(
                    &mut self.kernel,
                    std::slice::from_ref(event),
                )?;
            }
        }
        Ok(())
    }

    pub fn generate_manifest(&self) -> VerificationManifest {
        VerificationManifest {
            node_id: self.node_id,
            sequence_id: self.kernel.last_applied_sequence_id().unwrap_or(0),
            state_hash: self.kernel.state_hash(),
        }
    }
}

impl DeterministicState for ClusterNode {
    fn state_hash(&self) -> [u8; 32] {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.kernel.state_hash());
        bytes.extend_from_slice(&self.consensus.state_hash());
        bytes.extend_from_slice(&self.replication.state_hash());
        hash_bytes(&bytes)
    }
}
