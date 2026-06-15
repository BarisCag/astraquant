use astra_audit::verification::{ReplayVerifier, VerificationStatus};

#[test]
fn test_replay_parity_identical() {
    let verifier = ReplayVerifier { windows: vec![] };
    let hash = [42u8; 32];
    assert!(matches!(verifier.verify_parity(&hash, &hash), VerificationStatus::Passed));
}

#[test]
fn test_replay_parity_divergent() {
    let verifier = ReplayVerifier { windows: vec![] };
    let a = [1u8; 32];
    let b = [2u8; 32];
    assert!(matches!(verifier.verify_parity(&a, &b), VerificationStatus::Failed));
}
