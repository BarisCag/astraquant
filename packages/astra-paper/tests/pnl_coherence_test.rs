use astra_core::events::{AstraEvent, EventType};
use astra_core::marketdata::MarketTick;
use astra_core::types::{Price, Quantity};
use astra_paper::engine::PaperEngine;
use astra_paper::execution::{PaperExecutionEngine, FillModel};
use astra_paper::portfolio::PortfolioTracker;
use astra_paper::risk::{RiskEngine, RiskLimits};
use astra_paper::strategy::TwapStrategy;
use astra_paper::types::Side;

fn generate_synthetic_crash_data() -> Vec<AstraEvent> {
    let mut events = Vec::new();
    let mut price: i64 = 50_000_000_000; // 500 USD
    let mut timestamp_ns = 1_000_000_000_000;
    
    // 1 hour = 3600 seconds
    for i in 0..3600 {
        if i > 1800 && i <= 1860 {
            // Flash crash: drop 20% in 60s
            price -= 166_666_666;
        } else if i > 1860 && i <= 2400 {
            // Partial recovery
            price += 15_000_000;
        } else {
            if i % 2 == 0 { price += 1_000_000; } else { price -= 1_000_000; }
        }

        let tick = MarketTick {
            symbol: "BTCUSDT".to_string(),
            timestamp_ns,
            bid_price: Price((price - 100) as i64),
            ask_price: Price((price + 100) as i64),
            bid_quantity: Quantity(1_000_000),
            ask_quantity: Quantity(1_000_000),
        };

        let payload = astra_core::serialization::serialize_canonical(&tick).unwrap();
        events.push(AstraEvent::new_raw(timestamp_ns, i as u64, EventType::MarketTick, payload));
        timestamp_ns += 1_000_000_000;
    }
    events
}

#[test]
fn test_pnl_coherence_during_crash() {
    let events = generate_synthetic_crash_data();

    let portfolio = PortfolioTracker::new(100_000_000_000_000); // 100k
    let execution_engine = PaperExecutionEngine::new(FillModel::Slippage { n_trades: 5, slippage_bps: 2 });
    let risk_limits = RiskLimits {
        max_notional_per_symbol: 50_000_000_000_000,
        max_drawdown_usd: 50_000_000_000_000, // Very high to avoid trigger
        max_orders_per_second: 10,
    };
    let risk_engine = RiskEngine::new(risk_limits, 100_000_000_000_000);
    
    let twap = Box::new(TwapStrategy::new(
        "BTCUSDT".to_string(),
        Side::Buy,
        1_000_000_000,     // 10 BTC
        100_000_000,       // 1 BTC per slice
        60_000_000_000,    // 60s intervals -> 10 slices over 10 mins
        1_000_000_000_000, // start at very beginning
    ));

    let mut engine = PaperEngine::new(portfolio, execution_engine, risk_engine, twap as Box<dyn astra_paper::strategy::Strategy>);

    let mut min_nav = 100_000_000_000_000;
    let mut initial_nav = 0;
    let mut final_nav = 0;

    let dummy_hash = [0; 32];

    for (i, event) in events.iter().enumerate() {
        engine.process_event(event, &dummy_hash);
        
        let nav = engine.portfolio.total_equity(&engine.current_prices);
        if i == 0 { initial_nav = nav; }
        if i == events.len() - 1 { final_nav = nav; }
        if nav < min_nav { min_nav = nav; }
    }

    // TWAP strategy bought 10 BTC across the first 10 minutes. 
    // It held through the flash crash.
    // NAV should have dropped below initial.
    assert!(min_nav < initial_nav, "NAV did not drop during crash");
    assert!(final_nav > min_nav, "NAV did not recover after crash");
}
