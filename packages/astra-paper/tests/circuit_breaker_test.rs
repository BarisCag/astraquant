use astra_paper::portfolio::PortfolioTracker;
use astra_paper::risk::{RiskEngine, RiskLimits};
use astra_paper::types::{PaperOrder, Side, OrderType};
use std::time::Instant;

#[tokio::test]
async fn test_circuit_breaker_latency() {
    let limits = RiskLimits {
        max_notional_per_symbol: 10_000_000_000,
        max_drawdown_usd: 10_000_000_000,
        max_orders_per_second: 1000,
    };
    
    let portfolio = PortfolioTracker::new(100_000_000_000);
    let mut cb = RiskEngine::new(limits, 100_000_000_000);
    
    let current_price = 100_000_000; // 1 USD

    let order = PaperOrder {
        symbol: "BTCUSDT".to_string(),
        side: Side::Buy,
        order_type: OrderType::Market,
        quantity: 1_000_000, // Safe qty
    };

    let start = Instant::now();

    // Inject 1000 orders to hit the limit
    for i in 0..1000 {
        assert!(cb.filter_order(&order, 1000 + i, current_price, &portfolio, 100_000_000_000).is_ok());
    }

    // 1001st order should breach rate limit
    let breach_result = cb.filter_order(&order, 2001, current_price, &portfolio, 100_000_000_000);
    let engine_halted_at = Instant::now();

    assert!(breach_result.is_err());
    assert_eq!(breach_result.unwrap_err(), "Order rate limit breached! Kill switch activated.");
    assert!(cb.kill_switch_triggered);

    // Assert that detection and halt happened in < 50ms
    let elapsed = engine_halted_at.duration_since(start);
    assert!(elapsed < std::time::Duration::from_millis(50), "Latency was {:?} which exceeds 50ms", elapsed);
}
