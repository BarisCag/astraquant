use axum::{
    routing::{get, post},
    Router,
    middleware as axum_middleware,
};
use std::sync::Arc;

use crate::audit::AuditTrail;
use crate::auth::AuthManager;
use crate::config::Config;
use crate::demo::DemoMode;
use crate::middleware::{audit_log::audit_middleware, auth::auth_middleware, rate_limit::{rate_limit_middleware, RateLimiter}};
use crate::handlers::{admin, alm, market, portfolio, risk, treasury, ws};

#[derive(Clone)]
#[allow(dead_code)]
struct AppState {
    auth: Arc<AuthManager>,
    audit: Arc<AuditTrail>,
    demo: Arc<DemoMode>,
    rate_limiter: Arc<RateLimiter>,
}

pub fn build_router(config: Config, demo: DemoMode) -> Router {
    let auth = Arc::new(AuthManager::from_config(&config));
    let audit = Arc::new(AuditTrail::new());
    let rate_limiter = Arc::new(RateLimiter::new(&config));
    let demo_arc = Arc::new(demo);

    // Provide extensions to all requests: RateLimiter is needed by rate_limit_middleware
    let rate_limiter_clone = rate_limiter.clone();

    // Authenticated routes
    let api_routes = Router::new()
        // Market
        .route("/market/snapshot", get(market::get_market_snapshot))
        // Portfolio
        .route("/portfolio", get(portfolio::get_portfolio))
        .route("/portfolio/history", get(portfolio::get_portfolio_history))
        // Treasury
        .route("/treasury/cashflow", get(treasury::get_cashflow))
        .route("/treasury/exposure", get(treasury::get_exposure))
        // Risk
        .route("/risk/var", get(risk::get_var))
        .route("/risk/es", get(risk::get_es))
        .route("/risk/greeks", get(risk::get_greeks))
        .route("/risk/metrics", get(risk::get_metrics))
        // ALM
        .route("/alm/mismatch", get(alm::get_mismatch))
        .route("/alm/hedge/approve", post(alm::approve_hedge))
        // Admin
        .route("/admin/users", get(admin::get_users))
        .route("/admin/killswitch", post(admin::killswitch))
        .route("/admin/api-key/rotate", post(admin::rotate_api_key))
        
        .layer(axum_middleware::from_fn(rate_limit_middleware))
        .layer(axum_middleware::from_fn_with_state(auth.clone(), auth_middleware))
        .layer(axum_middleware::from_fn_with_state((auth.clone(), audit.clone()), |axum::extract::State((a, b)), req, next| audit_middleware(axum::extract::State(a), axum::extract::State(b), req, next)));

    // Health (unauthenticated, bypasses auth middleware)
    let health_route = Router::new()
        .route("/health", get(health_handler))
        .layer(axum_middleware::from_fn_with_state((auth.clone(), audit.clone()), |axum::extract::State((a, b)), req, next| audit_middleware(axum::extract::State(a), axum::extract::State(b), req, next)));

    // WebSocket (has its own query param auth)
    let ws_route = Router::new()
        .route("/ws/stream", get(ws::ws_handler))
        .with_state(auth.clone());

    Router::new()
        .merge(health_route)
        .merge(api_routes)
        .merge(ws_route)
        .layer(axum::Extension(rate_limiter_clone))
        .with_state(demo_arc)
}

#[derive(serde::Serialize)]
struct HealthResponse {
    status: String,
    mode: String,
    version: String,
}

async fn health_handler(
    axum::extract::State(demo): axum::extract::State<Arc<DemoMode>>,
) -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse {
        status: "ok".to_string(),
        mode: if demo.is_demo() { "demo".to_string() } else { "live".to_string() },
        version: "1.0".to_string(),
    })
}
