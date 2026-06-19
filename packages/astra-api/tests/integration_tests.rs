use astra_api::config::Config;
use astra_api::demo::DemoMode;
use astra_api::server::build_router;
use axum::{body::Body, http::Request};
use tower::ServiceExt; // for `oneshot` and `ready`

// Helper function to build a test router
fn test_router(demo: bool) -> axum::Router {
    let mut config = Config::load("config/default.toml");
    config.demo.enabled = demo;
    let hash_hex = hex::encode(blake3::hash(b"trader_key").as_bytes());
    config.api_keys.insert(hash_hex, "Trader".to_string());
    let demo_mode = DemoMode { enabled: demo };
    build_router(config, demo_mode)
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = test_router(false);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(body_json["status"], "ok");
    assert_eq!(body_json["mode"], "live");
}

#[tokio::test]
async fn test_market_snapshot() {
    let app = test_router(false);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/market/snapshot")
                .header("X-API-Key", "trader_key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(body_json.get("data").is_some());
}

#[tokio::test]
async fn test_unauthorized_access() {
    let app = test_router(false);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/market/snapshot")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_demo_mode_response() {
    let app = test_router(true);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(body_json["mode"], "demo");
}
