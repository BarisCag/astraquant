use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Trader,
    RiskManager,
    Treasurer,
    Auditor,
    Admin,
}

impl Role {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "trader" => Some(Role::Trader),
            "riskmanager" => Some(Role::RiskManager),
            "treasurer" => Some(Role::Treasurer),
            "auditor" => Some(Role::Auditor),
            "admin" => Some(Role::Admin),
            _ => None,
        }
    }
}

pub struct RoleEngine;

impl RoleEngine {
    pub fn can_access(role: &Role, endpoint: &str, method: &str) -> bool {
        if *role == Role::Admin {
            return true;
        }

        let is_get = method == "GET";
        let is_post = method == "POST";

        if endpoint.starts_with("/market/") && is_get {
            return true; // All roles
        }

        if endpoint.starts_with("/portfolio") && is_get {
            return true; // All roles
        }

        if endpoint.starts_with("/health") && is_get {
            return true; // All roles
        }

        if endpoint.starts_with("/risk/") && is_get {
            return true; // All roles
        }

        if endpoint.starts_with("/treasury/") && is_get {
            return matches!(role, Role::RiskManager | Role::Treasurer | Role::Auditor | Role::Admin);
        }

        if endpoint.starts_with("/alm/mismatch") && is_get {
            return matches!(role, Role::RiskManager | Role::Treasurer | Role::Auditor | Role::Admin);
        }

        if endpoint.starts_with("/alm/hedge/approve") && is_post {
            return matches!(role, Role::Treasurer | Role::Admin);
        }

        if endpoint.starts_with("/admin/killswitch") && is_post {
            return matches!(role, Role::RiskManager | Role::Admin);
        }

        if endpoint.starts_with("/admin/") && is_get {
            return matches!(role, Role::Admin);
        }

        if endpoint.starts_with("/audit/") && is_get {
            return matches!(role, Role::Auditor | Role::Admin);
        }
        
        // WS endpoint
        if endpoint.starts_with("/ws/stream") && is_get {
            return true; // Assume all can subscribe, actual topic filtering may apply later
        }

        false
    }
}
