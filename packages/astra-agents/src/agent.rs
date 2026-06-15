use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgentId(pub String);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AgentClass {
    LiquidityProvider,
    MarketMaker,
    Arbitrageur,
    TreasuryParticipant,
    ClearingMember,
    DistressedFund,
    PanicLiquidator,
    FundingDesk,
    SovereignParticipant,
    VolatilityReactiveParticipant,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgentState {
    pub agent_id: AgentId,
    pub class: AgentClass,
    pub capital: u64,
    pub inventory: BTreeMap<String, i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgentIntent {
    pub agent_id: AgentId,
    pub target_venue: u8,
    pub symbol: String,
    pub intent_type: String,
    pub size: u64,
    pub price: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BehaviorTransitionReason {
    SpreadThresholdExceeded,
    MarginStress,
    PanicDetected,
    LiquidityEvaporated,
    ArbitrageOpportunity,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgentDecision {
    pub intents: Vec<AgentIntent>,
    pub transition_reason: Option<BehaviorTransitionReason>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgentEvaluationWindow {
    pub sequence: u64,
    pub decisions: Vec<AgentDecision>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgentCheckpoint {
    pub sequence: u64,
    pub states: BTreeMap<AgentId, AgentState>,
}
