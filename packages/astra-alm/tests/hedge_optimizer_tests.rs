use astra_alm::hedge_optimizer::HedgeOptimizer;
use astra_alm::types::{ALMMismatchReport, Currency, TenorBucket};

#[test]
fn test_hedge_ratios_bounded() {
    let optimizer = HedgeOptimizer::new();
    let mismatches = vec![
        ALMMismatchReport {
            tenor: TenorBucket::FiveYear,
            currency: Currency::USD,
            asset_duration: 3.0,
            liability_duration: 7.0,
            duration_gap: -4.0,
            notional_gap: 0,
        }
    ];

    let ratios = optimizer.optimize(&mismatches, 1_000_000.0);
    for r in ratios {
        assert!(r.ratio >= 0.0 && r.ratio <= 1.0);
    }
}

#[test]
fn test_optimizer_determinism() {
    let optimizer = HedgeOptimizer::new();
    let mismatches = vec![
        ALMMismatchReport {
            tenor: TenorBucket::FiveYear,
            currency: Currency::USD,
            asset_duration: 3.0,
            liability_duration: 7.0,
            duration_gap: -4.0,
            notional_gap: 0,
        }
    ];

    let ratios1 = optimizer.optimize(&mismatches, 1_000_000.0);
    let ratios2 = optimizer.optimize(&mismatches, 1_000_000.0);

    assert_eq!(ratios1, ratios2);
}

#[test]
fn test_optimizer_reduces_cvar() {
    let optimizer = HedgeOptimizer::new();
    let mismatches = vec![
        ALMMismatchReport {
            tenor: TenorBucket::FiveYear,
            currency: Currency::USD,
            asset_duration: 3.0,
            liability_duration: 7.0,
            duration_gap: -4.0,
            notional_gap: 0,
        }
    ];

    let base_cvar = 1_000_000.0;
    
    // Initial CVaR without hedging (ratio = 0.0)
    let initial_ratios = vec![astra_alm::types::HedgeRatio {
        instrument: "HEDGE_INSTRUMENT_0".to_string(),
        ratio: 0.0,
        notional: 0,
    }];
    let initial_cvar = HedgeOptimizer::compute_portfolio_cvar(&initial_ratios, &mismatches, base_cvar);

    let optimal_ratios = optimizer.optimize(&mismatches, base_cvar);
    let optimal_cvar = HedgeOptimizer::compute_portfolio_cvar(&optimal_ratios, &mismatches, base_cvar);

    assert!(optimal_cvar <= initial_cvar);
}
