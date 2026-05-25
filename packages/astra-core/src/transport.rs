use crate::events::AstraEvent;
use crate::hashing::{hash_bytes, DeterministicState};
use crate::serialization::serialize_canonical;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransportPacket {
    Ping {
        node_id: u64,
    },
    ProposeEvent {
        node_id: u64,
        event: AstraEvent,
    },
    AppendEntries {
        term: u64,
        leader_id: u64,
        prev_log_index: u64,
        prev_log_hash: [u8; 32],
        entries: Vec<AstraEvent>,
        leader_commit: u64,
    },
    SnapshotTransfer {
        node_id: u64,
        snapshot_bytes: Vec<u8>,
    },
    VerificationManifest {
        node_id: u64,
        sequence_id: u64,
        state_hash: [u8; 32],
    },
}

impl DeterministicState for TransportPacket {
    fn state_hash(&self) -> [u8; 32] {
        hash_bytes(&serialize_canonical(self).unwrap())
    }
}
