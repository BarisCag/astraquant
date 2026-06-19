use crate::engine::ALMEngine;
use crate::types::{Currency, RateSensitivity, TenorBucket};

#[test]
fn test_alm_engine_replay_determinism() {
    let mut engine1 = ALMEngine::new();
    
    for i in 0..10 {
        engine1.add_asset(RateSensitivity {
            tenor: TenorBucket::FiveYear,
            currency: Currency::USD,
            modified_duration: 4.5,
            convexity: 20.0,
            notional: (i + 1) * 100_000,
        });
        
        engine1.add_liability(RateSensitivity {
            tenor: TenorBucket::FiveYear,
            currency: Currency::USD,
            modified_duration: 5.0,
            convexity: 25.0,
            notional: (i + 1) * 90_000,
        });
    }

    let report1 = engine1.compute_mismatch();
    let state_a = engine1.state_hash();

    // Rebuild identical ALMEngine from scratch
    let mut engine2 = ALMEngine::new();
    
    for i in 0..10 {
        engine2.add_asset(RateSensitivity {
            tenor: TenorBucket::FiveYear,
            currency: Currency::USD,
            modified_duration: 4.5,
            convexity: 20.0,
            notional: (i + 1) * 100_000,
        });
        
        engine2.add_liability(RateSensitivity {
            tenor: TenorBucket::FiveYear,
            currency: Currency::USD,
            modified_duration: 5.0,
            convexity: 25.0,
            notional: (i + 1) * 90_000,
        });
    }

    let report2 = engine2.compute_mismatch();
    let state_b = engine2.state_hash();

    assert_eq!(state_a, state_b);
    assert_eq!(report1.len(), report2.len());
}
