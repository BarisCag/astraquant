use astra_alm::event::ALMEvent;
use astra_alm::immunization::ImmunizationTracker;
use astra_alm::types::{ALMMismatchReport, Currency, TenorBucket};

#[test]
fn test_no_trigger_below_threshold() {
    let tracker = ImmunizationTracker::new();
    let report = ALMMismatchReport {
        tenor: TenorBucket::FiveYear,
        currency: Currency::USD,
        asset_duration: 5.0,
        liability_duration: 5.1,
        duration_gap: 0.10,
        notional_gap: 0,
    };
    
    let result = tracker.check(&report);
    assert!(result.is_none());
}

#[test]
fn test_trigger_above_threshold() {
    let tracker = ImmunizationTracker::new();
    let report = ALMMismatchReport {
        tenor: TenorBucket::FiveYear,
        currency: Currency::USD,
        asset_duration: 5.0,
        liability_duration: 5.3,
        duration_gap: 0.30,
        notional_gap: 0,
    };
    
    let result = tracker.check(&report);
    assert!(result.is_some());
    if let Some(ALMEvent::RebalancingRecommended { currency, tenor, duration_gap, .. }) = result {
        assert_eq!(currency, Currency::USD);
        assert_eq!(tenor, TenorBucket::FiveYear);
        assert_eq!(duration_gap, 0.30);
    } else {
        panic!("Wrong event type");
    }
}

#[test]
fn test_threshold_exactly_at_boundary() {
    let tracker = ImmunizationTracker::new();
    let report = ALMMismatchReport {
        tenor: TenorBucket::FiveYear,
        currency: Currency::USD,
        asset_duration: 5.0,
        liability_duration: 5.25,
        duration_gap: 0.25,
        notional_gap: 0,
    };
    
    let result = tracker.check(&report);
    assert!(result.is_none());
}
