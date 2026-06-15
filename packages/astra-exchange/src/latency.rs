use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LatencyEnvelope {
    pub delay_ns: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct VenueLatencyProfile {
    pub venue_id: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExecutionDelayWindow {
    pub window_size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DeterministicLatencyInjector {
    pub seed: u64,
}

impl LatencyEnvelope {
    pub fn new() -> Self {
        Default::default()
    }
}
impl VenueLatencyProfile {
    pub fn new() -> Self {
        Default::default()
    }
}
impl ExecutionDelayWindow {
    pub fn new() -> Self {
        Default::default()
    }
}
impl DeterministicLatencyInjector {
    pub fn new() -> Self {
        Default::default()
    }
}
