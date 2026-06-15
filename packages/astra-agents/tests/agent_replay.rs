use astra_agents::agent::{AgentClass, AgentId, AgentState};
use std::collections::BTreeMap;

#[test]
fn test_agent_serialization_deterministic() {
    let state = AgentState {
        agent_id: AgentId("MM1".to_string()),
        class: AgentClass::MarketMaker,
        capital: 1000000,
        inventory: BTreeMap::new(),
    };
    let bytes1 = bincode::serialize(&state).unwrap();
    let bytes2 = bincode::serialize(&state).unwrap();
    assert_eq!(bytes1, bytes2);
}
