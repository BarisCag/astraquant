use astra_scenarios::scenario::ExperimentParameterSet;
use astra_research::sweep::{ParameterSweep, SweepDimension};

#[test]
fn test_sweep_generation() {
    let dimensions = vec![
        SweepDimension::LiquidityCollapseSeverity { start_ppm: 0, end_ppm: 500_000, steps: 3 },
        SweepDimension::MarginMultiplier { start_ppm: 1_000_000, end_ppm: 2_000_000, steps: 2 },
    ];

    let sweep = ParameterSweep::new(dimensions);
    let plan = sweep.generate_plan(&ExperimentParameterSet::default());

    // 3 * 2 = 6 parameter sets
    assert_eq!(plan.parameter_sets.len(), 6);

    // Verify first set (0, 1_000_000)
    assert_eq!(plan.parameter_sets[0].liquidity_drop_ppm, 0);
    assert_eq!(plan.parameter_sets[0].margin_multiplier_ppm, 1_000_000);

    // Verify last set (500_000, 2_000_000)
    assert_eq!(plan.parameter_sets[5].liquidity_drop_ppm, 500_000);
    assert_eq!(plan.parameter_sets[5].margin_multiplier_ppm, 2_000_000);
}
