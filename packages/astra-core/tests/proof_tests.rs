use astra_core::audit::AuditEngine;
use astra_core::events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata};
use astra_core::exchange::ExchangeRuntime;
use astra_core::hashing::DeterministicState;
use astra_core::kernel::AstraKernel;
use astra_core::merkle::MerkleTree;
use astra_core::proof::StateTransitionProof;
use astra_core::replay::EventReducer;
use astra_core::risk::create_default_risk_engine;
use astra_core::runtime::StrategyRuntime;
use astra_core::symbolic::SymbolicReplayEngine;
use astra_core::types::{Money, Quantity};

fn create_kernel() -> AstraKernel {
    let limits =
        create_default_risk_engine(Money::new(10_000_000_000_000), Quantity::new(1_000_000_000));
    AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits)))
}

#[test]
fn test_formal_verification_system() {
    let mut kernel_a = create_kernel();

    let event = AstraEvent::new(
        1_700_000_000_000_000_000,
        1,
        EventType::MarketTick,
        vec![1, 2, 3],
        PayloadMetadata::new(PayloadEncoding::RawBytes, 1),
    );

    let pre_hash = kernel_a.state_hash();
    kernel_a.apply(&event).unwrap();
    let post_hash = kernel_a.state_hash();

    // 1. Proof verification
    let proof = StateTransitionProof::generate(pre_hash, &event, post_hash);
    assert!(proof.verify(&post_hash));

    // 2. Symbolic replay divergence check
    let events = vec![event.clone()];
    let mut clean_a = create_kernel();
    let mut clean_b = create_kernel();
    let has_diverged = SymbolicReplayEngine::detect_divergence(&mut clean_a, &mut clean_b, &events);
    assert!(!has_diverged);

    // 3. Merkle Root Check
    let leaves = vec![pre_hash, post_hash];
    let tree = MerkleTree::build(&leaves);
    let root = tree.root_hash().unwrap();
    assert!(AuditEngine::verify_merkle_root(&tree, &root));
}
