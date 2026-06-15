use astra_agents::ecology::{AgentEcology, EcologyOrchestrator};
use std::collections::BTreeMap;

#[test]
fn test_ecology_evaluation_ordering() {
    let ecology = AgentEcology { agents: BTreeMap::new() };
    let mut orchestrator = EcologyOrchestrator::new(ecology);
    let batch = orchestrator.evaluate_sequence(100);
    assert_eq!(batch.sequence, 100);
}
