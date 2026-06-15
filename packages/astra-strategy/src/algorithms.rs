use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MeanReversionStrategy {
    pub dummy: i64,
}

impl MeanReversionStrategy {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MomentumStrategy {
    pub dummy: i64,
}

impl MomentumStrategy {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LiquidityTakingStrategy {
    pub dummy: i64,
}

impl LiquidityTakingStrategy {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PassiveMarketMakingStrategy {
    pub dummy: i64,
}

impl PassiveMarketMakingStrategy {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SpreadCaptureStrategy {
    pub dummy: i64,
}

impl SpreadCaptureStrategy {
    pub fn new() -> Self {
        Default::default()
    }
}
