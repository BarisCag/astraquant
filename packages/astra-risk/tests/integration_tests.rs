use astra_risk::engine::RiskEngine;
use astra_risk::types::TraderRiskProfile;

#[test]
fn test_velocity_limit_rejection() {
    let mut risk = RiskEngine::new();
    risk.register_trader(TraderRiskProfile {
        trader_id: 1,
        max_position_notional: 1_000_000,
        max_order_quantity: 100,
        max_drawdown: 10_000,
        max_order_velocity: 3,
    });

    risk.increment_sequence(); // sequence 1
    assert!(risk.validate_order(1, 10, 100).is_ok());

    risk.increment_sequence(); // sequence 2
    assert!(risk.validate_order(1, 10, 100).is_ok());

    risk.increment_sequence(); // sequence 3
    assert!(risk.validate_order(1, 10, 100).is_ok());

    risk.increment_sequence(); // sequence 4
                               // Velocity limit of 3 exceeded in window
    let res = risk.validate_order(1, 10, 100);
    assert_eq!(
        res,
        Err(astra_risk::types::RiskViolation::MaxOrderVelocityExceeded)
    );
}

#[test]
fn test_drawdown_limit_rejection() {
    let mut risk = RiskEngine::new();
    risk.register_trader(TraderRiskProfile {
        trader_id: 1,
        max_position_notional: 1_000_000,
        max_order_quantity: 100,
        max_drawdown: 500,
        max_order_velocity: 10,
    });

    risk.increment_sequence();
    assert!(risk.validate_order(1, 10, 100).is_ok());

    // Apply a massive loss
    risk.apply_fill(1, 0, 0, -600);

    risk.increment_sequence();
    let res = risk.validate_order(1, 10, 100);
    assert_eq!(
        res,
        Err(astra_risk::types::RiskViolation::MaxDrawdownExceeded)
    );
}

#[test]
fn test_state_hash_determinism() {
    let mut risk1 = RiskEngine::new();
    risk1.register_trader(TraderRiskProfile {
        trader_id: 1,
        max_position_notional: 10_000,
        max_order_quantity: 10,
        max_drawdown: 100,
        max_order_velocity: 10,
    });
    risk1.increment_sequence();
    risk1.validate_order(1, 5, 50).unwrap();

    let mut risk2 = RiskEngine::new();
    risk2.register_trader(TraderRiskProfile {
        trader_id: 1,
        max_position_notional: 10_000,
        max_order_quantity: 10,
        max_drawdown: 100,
        max_order_velocity: 10,
    });
    risk2.increment_sequence();
    risk2.validate_order(1, 5, 50).unwrap();

    assert_eq!(risk1.state_hash(), risk2.state_hash());
}
