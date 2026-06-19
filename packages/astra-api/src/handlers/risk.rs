use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;

use crate::demo::DemoMode;
use crate::handlers::ApiResponse;

#[derive(Serialize, Deserialize, Clone)]
pub struct VarReport {
    pub var_99: f64,
    pub var_95: f64,
    pub method: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EsReport {
    pub es_975: f64,
    pub confidence: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Greeks {
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta: f64,
    pub rho: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CombinedMetrics {
    pub var: VarReport,
    pub es: EsReport,
    pub greeks: Greeks,
}

pub async fn get_var(
    State(demo): State<Arc<DemoMode>>,
) -> Json<ApiResponse<VarReport>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

    let report = VarReport {
        var_99: demo.sanitize_position(1_200_000) as f64,
        var_95: demo.sanitize_position(800_000) as f64,
        method: "parametric".to_string(),
    };

    Json(ApiResponse {
        data: report,
        timestamp_ns: now,
        mode: if demo.is_demo() { "demo".to_string() } else { "live".to_string() },
    })
}

pub async fn get_es(
    State(demo): State<Arc<DemoMode>>,
) -> Json<ApiResponse<EsReport>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

    let report = EsReport {
        es_975: demo.sanitize_position(1_500_000) as f64,
        confidence: 0.975,
    };

    Json(ApiResponse {
        data: report,
        timestamp_ns: now,
        mode: if demo.is_demo() { "demo".to_string() } else { "live".to_string() },
    })
}

pub async fn get_greeks(
    State(demo): State<Arc<DemoMode>>,
) -> Json<ApiResponse<Greeks>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

    let greeks = Greeks {
        delta: 0.55,
        gamma: 0.05,
        vega: 120.0,
        theta: -50.0,
        rho: 25.0,
    };

    Json(ApiResponse {
        data: greeks,
        timestamp_ns: now,
        mode: if demo.is_demo() { "demo".to_string() } else { "live".to_string() },
    })
}

pub async fn get_metrics(
    State(demo): State<Arc<DemoMode>>,
) -> Json<ApiResponse<CombinedMetrics>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

    let metrics = CombinedMetrics {
        var: VarReport {
            var_99: demo.sanitize_position(1_200_000) as f64,
            var_95: demo.sanitize_position(800_000) as f64,
            method: "parametric".to_string(),
        },
        es: EsReport {
            es_975: demo.sanitize_position(1_500_000) as f64,
            confidence: 0.975,
        },
        greeks: Greeks {
            delta: 0.55,
            gamma: 0.05,
            vega: 120.0,
            theta: -50.0,
            rho: 25.0,
        },
    };

    Json(ApiResponse {
        data: metrics,
        timestamp_ns: now,
        mode: if demo.is_demo() { "demo".to_string() } else { "live".to_string() },
    })
}
