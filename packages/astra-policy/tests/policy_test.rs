use astra_policy::engine::PolicyEngine;
use astra_policy::intervention::LiquidityInjectionFacility;
use astra_policy::policy::{PolicyAction, PolicyExecutionWindow};
use astra_policy::systemic::SystemicPropagationMetrics;

#[test]
fn test_liquidity_facility_activation() {
    let mut engine = PolicyEngine::new();
    
    let facility = LiquidityInjectionFacility {
        facility_id: 1,
        total_capacity: 10_000,
        injected_amount: 0,
        window: PolicyExecutionWindow {
            start_sequence: 100,
            end_sequence: 150,
        },
    };
    
    engine.interventions.liquidity_facilities.push(facility);
    
    let metrics = SystemicPropagationMetrics::new();
    
    // Evaluate sequence 99 (Before window)
    assert!(!engine.interventions.liquidity_facilities[0].is_active(99));
    
    // Evaluate sequence 100 (In window)
    assert!(engine.interventions.liquidity_facilities[0].is_active(100));
    
    // Evaluate sequence 150 (In window)
    assert!(engine.interventions.liquidity_facilities[0].is_active(150));
    
    // Evaluate sequence 151 (After window)
    assert!(!engine.interventions.liquidity_facilities[0].is_active(151));
}
