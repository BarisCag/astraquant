use axum::{

    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::audit::{AuditEntry, AuditTrail};
use crate::auth::AuthManager;

pub async fn audit_middleware(
    State(auth): State<Arc<AuthManager>>,
    State(audit): State<Arc<AuditTrail>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = std::time::Instant::now();
    let endpoint = req.uri().path().to_string();
    let method = req.method().as_str().to_string();

    // Check for auth header to extract pseudo api_key_hash
    let mut api_key_hash = [0u8; 32];
    if let Some(auth_val) = req.headers().get(axum::http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_val.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Some(claims) = auth.validate_jwt(token) {
                    // MOCK: using user_id hash
                    let hash = *blake3::hash(claims.sub.as_bytes()).as_bytes();
                    api_key_hash = hash;
                }
            }
        }
    } else if let Some(api_key_val) = req.headers().get("X-API-Key") {
        if let Ok(key_str) = api_key_val.to_str() {
            if let Some(key) = auth.validate_api_key(key_str) {
                api_key_hash = key.key_hash;
            }
        }
    }

    let request_body_hash = [0u8; 32]; // Simplification for demo
    
    let res = next.run(req).await;
    
    let processing_time_ms = start_time.elapsed().as_millis() as u64;
    let response_status = res.status().as_u16();
    let timestamp_ns = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

    let entry = AuditEntry {
        timestamp_ns,
        api_key_hash,
        endpoint,
        method,
        request_body_hash,
        response_status,
        processing_time_ms,
        chain_hash: [0u8; 32],
    };

    // Non-blocking log
    let audit_clone = audit.clone();
    tokio::spawn(async move {
        audit_clone.append(entry).await;
    });

    Ok(res)
}
