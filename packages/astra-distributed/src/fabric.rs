use serde::{Deserialize, Serialize};
use crate::partition::{ReplayShard, ReplayShardCertification};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayDispatchEnvelope {
    pub message_id: String,
    pub shard: ReplayShard,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayShardTransfer {
    pub transfer_id: String,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificationSyncEnvelope {
    pub node_id: String,
    pub certification: ReplayShardCertification,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayFabricProtocol;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DistributedReplayTransport;
