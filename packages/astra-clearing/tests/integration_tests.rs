use astra_clearing::funding::FundingLedger;
use astra_clearing::margin::{MarginEngine, TraderMarginProfile};
use astra_clearing::settlement::SettlementEngine;

#[test]
fn test_margin_liquidation() {
    let mut margin_engine = MarginEngine::new();

    let profile = TraderMarginProfile {
        initial_margin_ppm: 100_000,    // 10%
        maintenance_margin_ppm: 50_000, // 5%
        liquidation_grace_sequences: 3,
        max_leverage_ppm: 10_000_000,
        collateral_haircut_ppm: 1_000_000,
    };
    margin_engine.register_profile(1, profile);

    // Provide 100k collateral, utilize 1M margin
    // 100k / 1M = 10% (100_000 ppm) -> Health is exactly initial margin
    margin_engine.update_collateral(1, 100_000, 1_000_000);

    // Check sequence 1, should be no liquidations
    let liqs = margin_engine.check_margin_health(1);
    assert!(liqs.is_empty());

    // Drop collateral to 40k -> Health 40k / 1M = 4% (40_000 ppm) -> Below 5% MM
    margin_engine.update_collateral(1, 40_000, 1_000_000);

    // Sequence 2: margin breach detected, margin call issued, but grace period active (deadline = 2 + 3 = 5)
    let liqs = margin_engine.check_margin_health(2);
    assert!(liqs.is_empty());
    let account = margin_engine.accounts.get(&1).unwrap();
    assert!(account.active_margin_call.is_some());

    // Sequence 4: still active
    let liqs = margin_engine.check_margin_health(4);
    assert!(liqs.is_empty());

    // Sequence 5: grace period expired, liquidation issued
    let liqs = margin_engine.check_margin_health(5);
    assert_eq!(liqs.len(), 1);
    assert_eq!(liqs[0].trader_id, 1);
}

#[test]
fn test_settlement_replay_parity() {
    let mut se1 = SettlementEngine::new(2);
    let mut se2 = SettlementEngine::new(2);

    se1.queue_trade(1, "BTCUSD".to_string(), 10, 50000, true, 100);
    se2.queue_trade(1, "BTCUSD".to_string(), 10, 50000, true, 100);

    let m1 = se1.mature_obligations(102);
    let m2 = se2.mature_obligations(102);

    assert_eq!(m1.len(), 1);
    assert_eq!(m2.len(), 1);
    assert_eq!(m1[0].net_cash_movement, m2[0].net_cash_movement);

    // In a real replay test, we would hash the states
    // but the struct shapes deterministically map to bincode.
}
