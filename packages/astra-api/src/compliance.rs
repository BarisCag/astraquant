use crate::rbac::Role;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ControlActivity {
    pub control_id: String,
    pub description: String,
    pub frequency: String,
    pub responsible_role: Role,
    pub last_tested: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ExceptionSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Exception {
    pub exception_id: String,
    pub control_id: String,
    pub severity: ExceptionSeverity,
    pub description: String,
    pub remediation_status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Evidence {
    pub audit_trail_hash: [u8; 32],
    pub timestamp_ns: u64,
    pub description: String,
}
