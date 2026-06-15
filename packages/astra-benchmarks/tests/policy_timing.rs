use astra_benchmarks::research::InterventionTimingStudy;

#[test]
fn test_policy_timing_divergence() {
    let study = InterventionTimingStudy {
        study_id: "timing_study_alpha".to_string(),
        baseline_loss: 10_000,
        delayed_loss: 50_000,
        immediate_loss: 5_000,
    };

    // Assert deterministic penalty for delay
    assert!(study.delayed_loss > study.baseline_loss);
    assert!(study.immediate_loss < study.baseline_loss);
}
