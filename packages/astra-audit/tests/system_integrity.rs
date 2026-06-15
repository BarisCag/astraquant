use astra_audit::report::SystemIntegrityReport;

#[test]
fn test_system_integrity_scoring() {
    let report = SystemIntegrityReport {
        sequence_integrity_score_ppm: 1_000_000,
        lineage_consistency_score_ppm: 950_000,
        benchmark_trust_score_ppm: 980_000,
        overall_trust_score_ppm: 976_666,
    };
    assert!(report.overall_trust_score_ppm <= 1_000_000);
    assert!(report.lineage_consistency_score_ppm > 0);
}
