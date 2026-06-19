use astra_alm::engine::ALMEngine;
use astra_alm::types::{Currency, RateSensitivity, TenorBucket};

#[test]
fn test_mismatch_report_all_tenors() {
    let mut engine = ALMEngine::new();
    
    let tenors = vec![
        TenorBucket::Overnight, TenorBucket::OneWeek, TenorBucket::OneMonth,
        TenorBucket::ThreeMonth, TenorBucket::SixMonth, TenorBucket::OneYear,
        TenorBucket::TwoYear, TenorBucket::FiveYear, TenorBucket::TenYear
    ];
    
    for tenor in tenors {
        engine.add_asset(RateSensitivity {
            tenor,
            currency: Currency::USD,
            modified_duration: 3.0,
            convexity: 10.0,
            notional: 1_000_000,
        });
        
        engine.add_liability(RateSensitivity {
            tenor,
            currency: Currency::USD,
            modified_duration: 3.0,
            convexity: 10.0,
            notional: 1_000_000,
        });
    }
    
    let report = engine.compute_mismatch();
    assert_eq!(report.len(), 9);
}

#[test]
fn test_mismatch_report_all_currencies() {
    let mut engine = ALMEngine::new();
    
    let currencies = vec![
        Currency::USD, Currency::EUR, Currency::GBP, Currency::JPY, Currency::CHF
    ];
    
    for currency in currencies {
        engine.add_asset(RateSensitivity {
            tenor: TenorBucket::OneYear,
            currency,
            modified_duration: 1.0,
            convexity: 1.0,
            notional: 1_000_000,
        });
        
        engine.add_liability(RateSensitivity {
            tenor: TenorBucket::OneYear,
            currency,
            modified_duration: 1.0,
            convexity: 1.0,
            notional: 1_000_000,
        });
    }
    
    let report = engine.compute_mismatch();
    assert_eq!(report.len(), 5);
}

#[test]
fn test_duration_gap_calculation() {
    let mut engine = ALMEngine::new();
    
    engine.add_asset(RateSensitivity {
        tenor: TenorBucket::FiveYear,
        currency: Currency::USD,
        modified_duration: 3.0,
        convexity: 15.0,
        notional: 1_000_000_00,
    });
    
    engine.add_liability(RateSensitivity {
        tenor: TenorBucket::FiveYear,
        currency: Currency::USD,
        modified_duration: 7.0,
        convexity: 45.0,
        notional: 1_000_000_00,
    });
    
    let reports = engine.compute_mismatch();
    assert_eq!(reports.len(), 1);
    
    let report = &reports[0];
    assert_eq!(report.duration_gap, -4.0);
}

#[test]
fn test_state_hash_determinism() {
    let mut engine1 = ALMEngine::new();
    let mut engine2 = ALMEngine::new();
    
    let asset = RateSensitivity {
        tenor: TenorBucket::OneYear,
        currency: Currency::EUR,
        modified_duration: 1.0,
        convexity: 1.0,
        notional: 500_000,
    };
    
    engine1.add_asset(asset.clone());
    engine2.add_asset(asset);
    
    assert_eq!(engine1.state_hash(), engine2.state_hash());
}
