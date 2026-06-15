use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvariantAuditSummary {
    pub total_invariants_checked: u64,
    pub violations_detected: u64,
    pub critical_violations: u64,
    pub compliance_score_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayVerificationReport {
    pub total_windows_verified: u64,
    pub passed_windows: u64,
    pub failed_windows: u64,
    pub integrity_score_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemIntegrityReport {
    pub sequence_integrity_score_ppm: u64,
    pub lineage_consistency_score_ppm: u64,
    pub benchmark_trust_score_ppm: u64,
    pub overall_trust_score_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditReport {
    pub report_id: String,
    pub invariant_summary: InvariantAuditSummary,
    pub verification_report: ReplayVerificationReport,
    pub system_integrity: SystemIntegrityReport,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkIntegrityAudit {
    pub benchmark_id: String,
    pub certification_valid: bool,
    pub lineage_verified: bool,
    pub invariant_compliance_ppm: u64,
}
