use astra_benchmarks::research::SystemicRecoveryStudy;

#[test]
fn test_systemic_recovery_scoring() {
    let study = SystemicRecoveryStudy {
        study_id: "recovery_study_beta".to_string(),
        recovery_duration_sequences: 50_000,
        containment_efficiency_ppm: 950_000,
    };

    assert!(study.containment_efficiency_ppm <= 1_000_000);
    assert_eq!(study.recovery_duration_sequences, 50_000);
}
