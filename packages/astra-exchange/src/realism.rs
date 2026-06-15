use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LiquidityDepthModel {
    pub depth: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DeterministicSlippageEngine {
    pub slippage_ticks: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct QueuePressureTracker {
    pub pressure: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SpreadEvolutionModel {
    pub spread: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MarketImpactWindow {
    pub impact: u64,
}

impl LiquidityDepthModel {
    pub fn new() -> Self { Default::default() }
}
impl DeterministicSlippageEngine {
    pub fn new() -> Self { Default::default() }
}
impl QueuePressureTracker {
    pub fn new() -> Self { Default::default() }
}
impl SpreadEvolutionModel {
    pub fn new() -> Self { Default::default() }
}
impl MarketImpactWindow {
    pub fn new() -> Self { Default::default() }
}
