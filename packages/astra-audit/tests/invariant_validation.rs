use astra_audit::invariant::InvariantRegistry;

#[test]
fn test_sequence_monotonicity_pass() {
    let registry = InvariantRegistry::new();
    let sequences = vec![1, 2, 3, 4, 5];
    assert!(registry
        .evaluate_sequence_monotonicity(&sequences)
        .is_none());
}

#[test]
fn test_sequence_monotonicity_violation() {
    let registry = InvariantRegistry::new();
    let sequences = vec![1, 2, 5, 3, 6];
    let violation = registry.evaluate_sequence_monotonicity(&sequences);
    assert!(violation.is_some());
    assert_eq!(violation.unwrap().detected_at_sequence, 3);
}
