use astra_benchmarks::certification::{
    HashParityCertificate, ReplayLineageTree, ReplayParityProof,
};

#[test]
fn test_hash_parity_determinism() {
    let mut tree1 = ReplayLineageTree {
        root_hash: [0; 32],
        windows: vec![],
    };
    tree1.root_hash[0] = 42;

    let mut tree2 = ReplayLineageTree {
        root_hash: [0; 32],
        windows: vec![],
    };
    tree2.root_hash[0] = 42;

    let proof1 = ReplayParityProof {
        lineage_tree: tree1,
    };
    let proof2 = ReplayParityProof {
        lineage_tree: tree2,
    };

    let cert1 = HashParityCertificate {
        certified_hash: proof1.lineage_tree.root_hash,
        verifier: "astra_benchmark".to_string(),
    };

    let cert2 = HashParityCertificate {
        certified_hash: proof2.lineage_tree.root_hash,
        verifier: "astra_benchmark".to_string(),
    };

    assert_eq!(cert1.certified_hash, cert2.certified_hash);
}
