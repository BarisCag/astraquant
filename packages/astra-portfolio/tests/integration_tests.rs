use astra_portfolio::engine::PositionEngine;

#[test]
fn test_realized_pnl_correctness() {
    let mut engine = PositionEngine::new();
    let trader_id = 1;
    let symbol = "BTC/USD";

    // Buy 10 @ 100
    engine.apply_fill(trader_id, symbol, true, 10, 100);
    // Sell 5 @ 120
    engine.apply_fill(trader_id, symbol, false, 5, 120);

    let snapshot = engine.generate_snapshot(trader_id);
    assert_eq!(snapshot.total_realized_pnl, 100); // 5 * (120 - 100)
    assert_eq!(snapshot.net_exposure, 5 * 120); // 5 remaining * last mark price 120
}

#[test]
fn test_unrealized_pnl_correctness() {
    let mut engine = PositionEngine::new();
    let trader_id = 1;
    let symbol = "BTC/USD";

    // Buy 10 @ 100
    engine.apply_fill(trader_id, symbol, true, 10, 100);

    engine.update_mark_price(symbol, 150);

    let snapshot = engine.generate_snapshot(trader_id);
    assert_eq!(snapshot.total_unrealized_pnl, 500); // 10 * (150 - 100)
}

#[test]
fn test_position_flips() {
    let mut engine = PositionEngine::new();
    let trader_id = 1;
    let symbol = "ETH/USD";

    // Long 10 @ 100
    engine.apply_fill(trader_id, symbol, true, 10, 100);

    // Short 15 @ 150 -> Sells 10 @ 150 (Realized PnL = +500), new position Short 5 @ 150
    engine.apply_fill(trader_id, symbol, false, 15, 150);

    let snapshot = engine.generate_snapshot(trader_id);
    assert_eq!(snapshot.total_realized_pnl, 500);
    assert_eq!(snapshot.net_exposure, -5 * 150); // -750
    assert_eq!(snapshot.gross_exposure, 5 * 150); // 750

    // Buy 10 @ 100 -> Buys 5 @ 100 (Realized PnL = +250), new position Long 5 @ 100
    engine.apply_fill(trader_id, symbol, true, 10, 100);

    let snapshot = engine.generate_snapshot(trader_id);
    assert_eq!(snapshot.total_realized_pnl, 750); // 500 + 250
    assert_eq!(snapshot.net_exposure, 5 * 100); // 500
}

#[test]
fn test_multi_symbol_aggregation() {
    let mut engine = PositionEngine::new();
    let trader_id = 1;

    engine.apply_fill(trader_id, "AAPL", true, 10, 150);
    engine.apply_fill(trader_id, "MSFT", false, 5, 300);

    let snapshot = engine.generate_snapshot(trader_id);
    assert_eq!(snapshot.active_symbol_count, 2);
    assert_eq!(snapshot.gross_exposure, 1500 + 1500); // 3000
    assert_eq!(snapshot.net_exposure, 1500 - 1500); // 0
}

#[test]
fn test_deterministic_hash_equality() {
    let mut engine1 = PositionEngine::new();
    engine1.apply_fill(1, "AAPL", true, 10, 150);

    let mut engine2 = PositionEngine::new();
    engine2.apply_fill(1, "AAPL", true, 10, 150);

    assert_eq!(engine1.state_hash(), engine2.state_hash());
}
