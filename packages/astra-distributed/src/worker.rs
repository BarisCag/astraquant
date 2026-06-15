use serde::{Deserialize, Serialize};
use crate::partition::{ReplayShard, ReplayShardCertification};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShardExecutionResult {
    pub shard_id: String,
    pub status: String,
    pub certification: ReplayShardCertification,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerCertificationWindow {
    pub start: u64,
    pub end: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayExecutionNode {
    pub node_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayWorker;

impl ReplayWorker {
    pub fn execute_shard(_shard: &ReplayShard) -> ShardExecutionResult {
        ShardExecutionResult {
            shard_id: String::new(),
            status: String::new(),
            certification: ReplayShardCertification {
                shard_id: String::new(),
                terminal_hash: [0; 32],
                lineage_hash: [0; 32],
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DistributedReplayExecutor;
