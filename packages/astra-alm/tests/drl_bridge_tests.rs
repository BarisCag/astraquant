use astra_alm::drl_bridge::DRLBridge;

#[test]
fn test_determinism_1000_runs() {
    let bridge = DRLBridge::new();
    let exposure_vector = vec![1.0, 2.0, 3.0, 4.0];
    let prev_hash = [0u8; 32];
    
    let first = bridge.recommend(&exposure_vector, prev_hash);
    
    for _ in 0..1000 {
        let current = bridge.recommend(&exposure_vector, prev_hash);
        assert_eq!(first.hedge_ratio, current.hedge_ratio);
        assert_eq!(first.confidence, current.confidence);
        assert_eq!(first.policy_version_hash, current.policy_version_hash);
        assert_eq!(first.target_notional, current.target_notional);
    }
}

#[test]
fn test_policy_hash_consistent() {
    let bridge = DRLBridge::new();
    let expected_hash = blake3::hash(b"stub_policy_v1");
    let rec = bridge.recommend(&[0.0], [0u8; 32]);
    assert_eq!(rec.policy_version_hash, *expected_hash.as_bytes());
}

#[test]
fn test_returns_valid_recommendation() {
    let bridge = DRLBridge::new();
    let rec = bridge.recommend(&[0.0], [0u8; 32]);
    assert_eq!(rec.hedge_ratio, 0.5);
    assert_eq!(rec.confidence, 0.75);
    assert_eq!(rec.instrument, "STUB_HEDGE_INSTRUMENT");
    assert_eq!(rec.target_notional, 0);
}
