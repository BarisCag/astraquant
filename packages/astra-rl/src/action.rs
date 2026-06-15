use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiscreteAction {
    WidenSpread,
    ReduceInventory,
    HaltLiquidity,
    InjectPassiveOrders,
    ActivateMarginDefense,
    ReduceExposure,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionConstraint {
    pub max_frequency: u64,
    pub cooling_period: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyActionSpace {
    pub available_actions: Vec<DiscreteAction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub sequence: u64,
    pub action: DiscreteAction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanonicalPolicyTable {
    pub mappings: BTreeMap<[u8; 32], DiscreteAction>,
}
