use crate::action::DiscreteAction;
use crate::observation::ObservationVector;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrajectoryWindow {
    pub observations: Vec<ObservationVector>,
    pub actions: Vec<DiscreteAction>,
    pub rewards: Vec<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpisodeManifest {
    pub episode_id: String,
    pub total_sequences: u64,
    pub trajectory_hash: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatasetCertification {
    pub signature: [u8; 32],
    pub lineage_hash: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayTrainingCorpus {
    pub episodes: Vec<EpisodeManifest>,
    pub certification: DatasetCertification,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayDatasetBuilder {}

impl Default for ReplayDatasetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplayDatasetBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn push_trajectory(&mut self, _window: TrajectoryWindow) {
        // dummy implementation
    }
}
