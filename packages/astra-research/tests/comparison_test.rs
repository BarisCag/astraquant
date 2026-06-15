use astra_research::comparison::{ExperimentComparison, ReplayDiff};
use astra_research::experiment::{ExperimentRun, SystemicMetrics};
use astra_scenarios::scenario::ExperimentParameterSet;

#[test]
fn test_experiment_comparison_identical() {
    let run_a = ExperimentRun {
        run_id: "run_a".to_string(),
        parameter_set: ExperimentParameterSet::default(),
        initial_seed: 42,
        final_state_hash: "hash_42_xyz".to_string(),
        replay_certification_hash: "cert_abc".to_string(),
        systemic_metrics: SystemicMetrics {
            experiment_replay_integrity_ppm: 1_000_000,
            systemic_variance_score_ppm: 0,
            liquidity_resilience_score_ppm: 1_000_000,
            recovery_success_score_ppm: 1_000_000,
            infrastructure_stability_score_ppm: 1_000_000,
        },
    };

    let run_b = run_a.clone();
    
    let diff = ExperimentComparison::compare(&run_a, &run_b);
    assert_eq!(diff.event_divergence_count, 0);
    assert_eq!(diff.sequence_divergence, 0);
    assert!(diff.final_hash_match);
}

#[test]
fn test_experiment_comparison_divergent() {
    let run_a = ExperimentRun {
        run_id: "run_a".to_string(),
        parameter_set: ExperimentParameterSet::default(),
        initial_seed: 42,
        final_state_hash: "hash_42_xyz".to_string(),
        replay_certification_hash: "cert_abc".to_string(),
        systemic_metrics: SystemicMetrics {
            experiment_replay_integrity_ppm: 1_000_000,
            systemic_variance_score_ppm: 0,
            liquidity_resilience_score_ppm: 1_000_000,
            recovery_success_score_ppm: 1_000_000,
            infrastructure_stability_score_ppm: 1_000_000,
        },
    };

    let mut run_b = run_a.clone();
    run_b.final_state_hash = "hash_42_diff".to_string();
    
    let diff = ExperimentComparison::compare(&run_a, &run_b);
    assert_eq!(diff.event_divergence_count, 1);
    assert_eq!(diff.sequence_divergence, 1);
    assert!(!diff.final_hash_match);
}
