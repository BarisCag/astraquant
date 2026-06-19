use astra_alm::engine::ALMEngine;
use astra_alm::event::ALMEvent;
use astra_alm::immunization::ImmunizationTracker;
use astra_alm::types::{Currency, RateSensitivity, TenorBucket};

#[test]
fn test_rate_shock_200bps() {
    let mut engine = ALMEngine::new();
    let tracker = ImmunizationTracker::new();

    // Initial State
    engine.add_asset(RateSensitivity {
        tenor: TenorBucket::FiveYear,
        currency: Currency::USD,
        modified_duration: 3.0,
        convexity: 0.0,
        notional: 1_000_000,
    });
    
    engine.add_liability(RateSensitivity {
        tenor: TenorBucket::FiveYear,
        currency: Currency::USD,
        modified_duration: 7.0,
        convexity: 0.0,
        notional: 1_000_000,
    });

    let reports = engine.compute_mismatch();
    let events = tracker.check_all(&reports);
    
    assert_eq!(events.len(), 1);
    if let ALMEvent::RebalancingRecommended { duration_gap, .. } = &events[0] {
        assert_eq!(*duration_gap, -4.0);
    } else {
        panic!("Expected RebalancingRecommended");
    }

    // Apply +200bps shock (yield from 0.05 to 0.07)
    let mut shocked_engine = ALMEngine::new();
    let shock_factor = 1.05 / 1.07;
    
    shocked_engine.add_asset(RateSensitivity {
        tenor: TenorBucket::FiveYear,
        currency: Currency::USD,
        modified_duration: 3.0 * shock_factor, // ≈ 2.9439
        convexity: 0.0,
        notional: 1_000_000,
    });
    
    shocked_engine.add_liability(RateSensitivity {
        tenor: TenorBucket::FiveYear,
        currency: Currency::USD,
        modified_duration: 7.0 * shock_factor, // ≈ 6.8691
        convexity: 0.0,
        notional: 1_000_000,
    });

    let shocked_reports = shocked_engine.compute_mismatch();
    let shocked_events = tracker.check_all(&shocked_reports);
    
    assert_eq!(shocked_events.len(), 1);
    if let ALMEvent::RebalancingRecommended { duration_gap, .. } = &shocked_events[0] {
        let expected_gap = (3.0 * shock_factor) - (7.0 * shock_factor);
        assert!((duration_gap - expected_gap).abs() < 1e-6);
        assert!(duration_gap.abs() > 0.25);
    } else {
        panic!("Expected RebalancingRecommended after shock");
    }
}

#[test]
fn test_multi_currency_shock() {
    let mut engine = ALMEngine::new();
    let tracker = ImmunizationTracker::new();
    let currencies = vec![
        Currency::USD, Currency::EUR, Currency::GBP, Currency::JPY, Currency::CHF
    ];

    for c in &currencies {
        engine.add_asset(RateSensitivity {
            tenor: TenorBucket::TenYear,
            currency: *c,
            modified_duration: 2.0,
            convexity: 0.0,
            notional: 1_000_000,
        });
        
        engine.add_liability(RateSensitivity {
            tenor: TenorBucket::TenYear,
            currency: *c,
            modified_duration: 9.0,
            convexity: 0.0,
            notional: 1_000_000,
        });
    }

    let reports = engine.compute_mismatch();
    let events = tracker.check_all(&reports);
    
    assert_eq!(events.len(), 5);
}
