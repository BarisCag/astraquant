use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;

use crate::demo::DemoMode;
use crate::handlers::ApiResponse;

#[derive(Serialize, Deserialize, Clone)]
pub struct CashflowForecast {
    pub forecast: Vec<i64>, // 30 days
    pub total_inflow: i64,
    pub total_outflow: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FxExposureEntry {
    pub pair: String,
    pub tenor: String,
    pub amount: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FxExposure {
    pub fx_exposure: Vec<FxExposureEntry>,
}

pub async fn get_cashflow(
    State(demo): State<Arc<DemoMode>>,
) -> Json<ApiResponse<CashflowForecast>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

    let forecast = CashflowForecast {
        forecast: vec![0; 30],
        total_inflow: demo.sanitize_position(10_000_000),
        total_outflow: demo.sanitize_position(8_000_000),
    };

    Json(ApiResponse {
        data: forecast,
        timestamp_ns: now,
        mode: if demo.is_demo() { "demo".to_string() } else { "live".to_string() },
    })
}

pub async fn get_exposure(
    State(demo): State<Arc<DemoMode>>,
) -> Json<ApiResponse<FxExposure>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

    let exposure = FxExposure {
        fx_exposure: vec![
            FxExposureEntry {
                pair: "EUR/USD".to_string(),
                tenor: "1M".to_string(),
                amount: demo.sanitize_position(5_000_000),
            }
        ],
    };

    Json(ApiResponse {
        data: exposure,
        timestamp_ns: now,
        mode: if demo.is_demo() { "demo".to_string() } else { "live".to_string() },
    })
}
