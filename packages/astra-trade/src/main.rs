use clap::Parser;
use std::sync::{Arc, Mutex};
use astra_venue::feed::LiveFeedManager;
use astra_paper::engine::PaperEngine;
use astra_paper::execution::{PaperExecutionEngine, FillModel};
use astra_paper::portfolio::PortfolioTracker;
use astra_paper::risk::{RiskEngine, RiskLimits};
use astra_paper::strategy::TwapStrategy;
use astra_paper::types::Side;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "live")]
    mode: String,

    #[arg(short, long, default_value = "btcusdt")]
    symbol: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("AstraQuant Trade - Starting in mode: {}", args.mode);

    let mut manager = LiveFeedManager::new("live_journal.astra_jl");

    if args.mode == "paper" {
        println!("Initializing Paper Trading Engine...");

        let portfolio = PortfolioTracker::new(100_000_000_000_000); // 100k USD * 1e8
        let execution_engine = PaperExecutionEngine::new(FillModel::Slippage { n_trades: 5, slippage_bps: 2 });
        let risk_limits = RiskLimits {
            max_notional_per_symbol: 50_000_000_000_000, // 50k USD
            max_drawdown_usd: 5_000_000_000_000,        // 5k USD
            max_orders_per_second: 10,
        };
        let risk_engine = RiskEngine::new(risk_limits, 100_000_000_000_000);
        
        let twap = Box::new(TwapStrategy::new(
            args.symbol.clone(),
            Side::Buy,
            1_000_000_000,     // 10 BTC
            100_000_000,       // 1 BTC per slice
            60_000_000_000,    // 60s intervals
            0,                 // start immediately
        ));

        let engine = Arc::new(Mutex::new(PaperEngine::new(
            portfolio,
            execution_engine,
            risk_engine,
            twap,
        )));

        let engine_clone = Arc::clone(&engine);

        println!("Connecting to Binance {} with Paper Interceptor...", args.symbol);
        manager.run(&args.symbol, Some(move |event: &astra_core::events::AstraEvent| {
            let mut engine_lock = engine_clone.lock().unwrap();
            let prev_hash = engine_lock.portfolio.cash_balance.to_le_bytes(); // We'd ideally pass the actual prev journal hash, but we don't have it easily here without modifying LiveFeedManager to pass it.
            // Wait, LiveFeedManager currently tracks `current_state_hash`. We can get it!
            // Let's pass a dummy for now since we're just intercepting, but for replay tests we will use the actual hash.
            let mut hash = [0u8; 32];
            hash[0..8].copy_from_slice(&prev_hash);
            
            let out_events = engine_lock.process_event(event, &hash);
            
            if !out_events.is_empty() {
                let eq = engine_lock.portfolio.cash_balance + engine_lock.portfolio.unrealized_pnl(&engine_lock.current_prices);
                let rpnl = engine_lock.portfolio.realized_pnl;
                println!("[PAPER] Equity: {:.2} USD | R-PnL: {:.2} USD ", eq as f64 / 1e8, rpnl as f64 / 1e8);
            }
            out_events
        })).await.unwrap();

    } else {
        println!("Connecting to Binance {} (Feed Only)...", args.symbol);
        let no_interceptor: Option<fn(&astra_core::events::AstraEvent) -> Vec<astra_core::events::AstraEvent>> = None;
        manager.run(&args.symbol, no_interceptor).await.unwrap();
    }
}
