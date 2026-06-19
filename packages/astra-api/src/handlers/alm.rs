use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;

use crate::demo::DemoMode;
use crate::handlers::ApiResponse;

#[derive(Serialize, Deserialize, Clone)]
pub struct ALMMismatchReport {
    pub tenor: String,
    pub currency: String,
    pub asset_duration: f64,
    pub liability_duration: f64,
    pub duration_gap: f64,
    pub notional_gap: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MismatchResponse {
    pub reports: Vec<ALMMismatchReport>,
}

#[derive(Deserialize)]
pub struct ApproveHedgeRequest {
    pub recommendation_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ApproveHedgeResponse {
    pub approved: bool,
    pub timestamp_ns: u64,
}

pub async fn get_mismatch(
    State(demo): State<Arc<DemoMode>>,
) -> Json<ApiResponse<MismatchResponse>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

    let report = ALMMismatchReport {
        tenor: "FiveYear".to_string(),
        currency: "USD".to_string(),
        asset_duration: 3.0,
        liability_duration: 7.0,
        duration_gap: -4.0,
        notional_gap: demo.sanitize_position(0),
    };

    let res = MismatchResponse {
        reports: vec![report],
    };

    Json(ApiResponse {
        data: res,
        timestamp_ns: now,
        mode: if demo.is_demo() { "demo".to_string() } else { "live".to_string() },
    })
}

pub async fn approve_hedge(
    State(demo): State<Arc<DemoMode>>,
    Json(_payload): Json<ApproveHedgeRequest>,
) -> Json<ApiResponse<ApproveHedgeResponse>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

    let res = ApproveHedgeResponse {
        approved: true,
        timestamp_ns: now,
    };

    Json(ApiResponse {
        data: res,
        timestamp_ns: now,
        mode: if demo.is_demo() { "demo".to_string() } else { "live".to_string() },
    })
}
