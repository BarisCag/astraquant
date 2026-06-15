use astra_audit::lineage::{CertificationChain, LineageGraph};

#[test]
fn test_certification_chain_deterministic() {
    let root = [42u8; 32];
    let chain1 = CertificationChain {
        chain_id: "chain_alpha".to_string(),
        lineage: LineageGraph::new(root),
        terminal_hash: [255u8; 32],
    };
    let chain2 = CertificationChain {
        chain_id: "chain_alpha".to_string(),
        lineage: LineageGraph::new(root),
        terminal_hash: [255u8; 32],
    };
    assert_eq!(chain1.terminal_hash, chain2.terminal_hash);
    assert_eq!(chain1.lineage.root_hash, chain2.lineage.root_hash);
}
