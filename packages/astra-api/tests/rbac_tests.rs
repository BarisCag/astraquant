use astra_api::rbac::{Role, RoleEngine};

#[test]
fn test_trader_cannot_access_treasury() {
    assert!(!RoleEngine::can_access(&Role::Trader, "/treasury/cashflow", "GET"));
}

#[test]
fn test_treasurer_can_approve_hedge() {
    assert!(RoleEngine::can_access(&Role::Treasurer, "/alm/hedge/approve", "POST"));
}

#[test]
fn test_auditor_cannot_post() {
    assert!(!RoleEngine::can_access(&Role::Auditor, "/alm/hedge/approve", "POST"));
    assert!(!RoleEngine::can_access(&Role::Auditor, "/admin/killswitch", "POST"));
}

#[test]
fn test_admin_full_access() {
    assert!(RoleEngine::can_access(&Role::Admin, "/treasury/cashflow", "GET"));
    assert!(RoleEngine::can_access(&Role::Admin, "/alm/hedge/approve", "POST"));
    assert!(RoleEngine::can_access(&Role::Admin, "/admin/killswitch", "POST"));
    assert!(RoleEngine::can_access(&Role::Admin, "/portfolio", "GET"));
}

#[test]
fn test_risk_manager_killswitch() {
    assert!(RoleEngine::can_access(&Role::RiskManager, "/admin/killswitch", "POST"));
}

#[test]
fn test_trader_market_access() {
    assert!(RoleEngine::can_access(&Role::Trader, "/market/snapshot", "GET"));
    assert!(RoleEngine::can_access(&Role::Trader, "/portfolio", "GET"));
    assert!(RoleEngine::can_access(&Role::Trader, "/risk/var", "GET"));
}
