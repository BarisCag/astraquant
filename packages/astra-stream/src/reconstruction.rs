use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct HistoricalReplayWindow {
    pub _dummy: u64,
}

impl HistoricalReplayWindow {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CanonicalMarketSession {
    pub _dummy: u64,
}

impl CanonicalMarketSession {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ReplaySequenceBoundary {
    pub _dummy: u64,
}

impl ReplaySequenceBoundary {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MarketSessionPartition {
    pub _dummy: u64,
}

impl MarketSessionPartition {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TradeQuoteReconstruction {
    pub _dummy: u64,
}

impl TradeQuoteReconstruction {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NormalizedVenueReplay {
    pub _dummy: u64,
}

impl NormalizedVenueReplay {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ReplayGapWindow {
    pub _dummy: u64,
}

impl ReplayGapWindow {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SessionLineageTrace {
    pub _dummy: u64,
}

impl SessionLineageTrace {
    pub fn dummy_method(&self) -> u64 {
        Default::default()
    }
}
