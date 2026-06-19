use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;

use crate::demo::DemoMode;
use crate::handlers::ApiResponse;

#[derive(Serialize, Deserialize, Clone)]
pub struct MarketSnapshot {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp_ns: u64,
}

pub async fn get_market_snapshot(
    State(demo): State<Arc<DemoMode>>,
) -> Json<ApiResponse<MarketSnapshot>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

    let snapshot = MarketSnapshot {
        symbol: demo.sanitize_instrument("BTC-USD"),
        price: 95000.0,
        volume: 120.5,
        timestamp_ns: now,
    };

    Json(ApiResponse {
        data: snapshot,
        timestamp_ns: now,
        mode: if demo.is_demo() { "demo".to_string() } else { "live".to_string() },
    })
}
