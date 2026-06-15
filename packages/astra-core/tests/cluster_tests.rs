use astra_core::cluster::ClusterNode;
use astra_core::events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata};
use astra_core::exchange::ExchangeRuntime;
use astra_core::kernel::AstraKernel;
use astra_core::risk::create_default_risk_engine;
use astra_core::runtime::StrategyRuntime;
use astra_core::transport::TransportPacket;
use astra_core::types::{Money, Quantity};
use astra_core::verification::verify_cluster_hashes;

fn create_kernel() -> AstraKernel {
    let limits =
        create_default_risk_engine(Money::new(10_000_000_000_000), Quantity::new(1_000_000_000));
    AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits)))
}

#[test]
fn test_cluster_consensus_replication() {
    let mut node_a = ClusterNode::new(1, create_kernel());
    let mut node_b = ClusterNode::new(2, create_kernel());
    let mut node_c = ClusterNode::new(3, create_kernel());

    let event = AstraEvent::new(
        1_700_000_000_000_000_000,
        1,
        EventType::MarketTick,
        vec![1, 2, 3],
        PayloadMetadata::new(PayloadEncoding::RawBytes, 1),
    );

    let packet = TransportPacket::AppendEntries {
        term: 1,
        leader_id: 1,
        prev_log_index: 0,
        prev_log_hash: [0; 32],
        entries: vec![event],
        leader_commit: 1,
    };

    // Replicate globally and perfectly deterministically
    node_a.receive_packet(&packet).unwrap();
    node_b.receive_packet(&packet).unwrap();
    node_c.receive_packet(&packet).unwrap();

    let manifest_a = node_a.generate_manifest();
    let manifest_b = node_b.generate_manifest();
    let manifest_c = node_c.generate_manifest();

    assert!(verify_cluster_hashes(&[manifest_a, manifest_b, manifest_c]));
}
