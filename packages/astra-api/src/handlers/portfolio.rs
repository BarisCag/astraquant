use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;

use crate::demo::DemoMode;
use crate::handlers::ApiResponse;

#[derive(Serialize, Deserialize, Clone)]
pub struct Position {
    pub symbol: String,
    pub notional: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PortfolioSnapshot {
    pub positions: Vec<Position>,
    pub total_pnl: i64,
    pub currency: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PortfolioHistory {
    pub snapshots: Vec<PortfolioSnapshot>,
}

pub async fn get_portfolio(
    State(demo): State<Arc<DemoMode>>,
) -> Json<ApiResponse<PortfolioSnapshot>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

    let snapshot = PortfolioSnapshot {
        positions: vec![
            Position {
                symbol: demo.sanitize_instrument("BTC-USD"),
                notional: demo.sanitize_position(5_250_000),
            },
            Position {
                symbol: demo.sanitize_instrument("ETH-USD"),
                notional: demo.sanitize_position(1_750_000),
            },
        ],
        total_pnl: demo.sanitize_pnl(125_450),
        currency: "USD".to_string(),
    };

    Json(ApiResponse {
        data: snapshot,
        timestamp_ns: now,
        mode: if demo.is_demo() { "demo".to_string() } else { "live".to_string() },
    })
}

pub async fn get_portfolio_history(
    State(demo): State<Arc<DemoMode>>,
) -> Json<ApiResponse<PortfolioHistory>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

    let snapshot = PortfolioSnapshot {
        positions: vec![
            Position {
                symbol: demo.sanitize_instrument("BTC-USD"),
                notional: demo.sanitize_position(5_250_000),
            },
        ],
        total_pnl: demo.sanitize_pnl(125_450),
        currency: "USD".to_string(),
    };

    let history = PortfolioHistory {
        snapshots: vec![snapshot],
    };

    Json(ApiResponse {
        data: history,
        timestamp_ns: now,
        mode: if demo.is_demo() { "demo".to_string() } else { "live".to_string() },
    })
}
