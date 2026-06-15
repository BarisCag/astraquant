use axum::{response::Json, routing::get, Router};
use serde_json::{json, Value};
use std::fs;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/datasets", get(get_datasets))
        .route("/calibration", get(get_calibration))
        .route("/counterfactuals", get(get_counterfactuals))
        .route("/health", get(|| async { Json(json!({"status": "ok"})) }));

    let port = std::env::var("ASTRA_HTTP_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    println!("Astra Research API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_datasets() -> Json<Value> {
    // Return mock datasets list, or scan directory if needed
    Json(json!([
        { "id": "flash_crash_2010", "name": "2010 Flash Crash", "events": 1000 },
        { "id": "lehman_2008", "name": "2008 Lehman Collapse", "events": 1200 },
        { "id": "covid_2020", "name": "2020 COVID Crash", "events": 800 }
    ]))
}

async fn get_calibration() -> Json<Value> {
    let path = "research_outputs/calibration_results.json";
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(json) = serde_json::from_str::<Value>(&content) {
            return Json(json);
        }
    }
    Json(json!({"error": "calibration results not found"}))
}

async fn get_counterfactuals() -> Json<Value> {
    let path = "research_outputs/behavioral_counterfactual_delta.json";
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(json) = serde_json::from_str::<Value>(&content) {
            return Json(json);
        }
    }
    // Fallback to Phase 13 counterfactual matrix
    let fallback = "research_outputs/counterfactual_matrix.json";
    if let Ok(content) = fs::read_to_string(fallback) {
        if let Ok(json) = serde_json::from_str::<Value>(&content) {
            return Json(json);
        }
    }
    Json(json!({"error": "counterfactual results not found"}))
}
