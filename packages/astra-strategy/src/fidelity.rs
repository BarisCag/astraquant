use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StrategyReplayVerifier {
    pub dummy: i64,
}

impl StrategyReplayVerifier {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PortfolioParityValidator {
    pub dummy: i64,
}

impl PortfolioParityValidator {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExecutionReplayCertification {
    pub dummy: i64,
}

impl ExecutionReplayCertification {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StrategyLineageProof {
    pub dummy: i64,
}

impl StrategyLineageProof {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PortfolioHashWindow {
    pub dummy: i64,
}

impl PortfolioHashWindow {
    pub fn new() -> Self {
        Default::default()
    }
}
