use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SequenceRange {
    pub start_sequence: u64,
    pub end_sequence: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayPartition {
    pub partition_id: String,
    pub range: SequenceRange,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayShard {
    pub shard_id: String,
    pub partition: ReplayPartition,
    pub expected_hash: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartitionWindow {
    pub start_time: u64,
    pub end_time: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DistributedReplayManifest {
    pub manifest_id: String,
    pub shards: Vec<ReplayShard>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayShardCertification {
    pub shard_id: String,
    pub terminal_hash: [u8; 32],
    pub lineage_hash: [u8; 32],
}
