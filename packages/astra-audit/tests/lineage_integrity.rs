use astra_audit::lineage::{LineageGraph, ReplayLineageNode};

#[test]
fn test_lineage_continuity_valid() {
    let root = [0u8; 32];
    let mut graph = LineageGraph::new(root);
    graph.add_node(ReplayLineageNode {
        sequence_id: 1000,
        state_hash: [1u8; 32],
        parent_hash: root,
    });
    graph.add_node(ReplayLineageNode {
        sequence_id: 2000,
        state_hash: [2u8; 32],
        parent_hash: [1u8; 32],
    });
    assert!(graph.verify_continuity());
}

#[test]
fn test_lineage_continuity_broken() {
    let root = [0u8; 32];
    let mut graph = LineageGraph::new(root);
    graph.add_node(ReplayLineageNode {
        sequence_id: 1000,
        state_hash: [1u8; 32],
        parent_hash: root,
    });
    graph.add_node(ReplayLineageNode {
        sequence_id: 2000,
        state_hash: [2u8; 32],
        parent_hash: [99u8; 32], // broken link
    });
    assert!(!graph.verify_continuity());
}
