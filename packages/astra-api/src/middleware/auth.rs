use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::auth::AuthManager;
use crate::rbac::{Role, RoleEngine};

pub async fn auth_middleware(
    State(auth): State<Arc<AuthManager>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().to_string();
    let method = req.method().as_str().to_string();

    // Skip auth for health
    if path == "/health" {
        return Ok(next.run(req).await);
    }
    
    // WS auth is handled in handler via query param
    if path == "/ws/stream" {
        return Ok(next.run(req).await);
    }

    let auth_header = req.headers().get(header::AUTHORIZATION);
    let api_key_header = req.headers().get("X-API-Key");

    let mut token_claims = None;
    let mut _api_key_role = None;

    if let Some(auth_val) = auth_header {
        if let Ok(auth_str) = auth_val.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Some(claims) = auth.validate_jwt(token) {
                    token_claims = Some(claims);
                }
            }
        }
    } else if let Some(api_key_val) = api_key_header {
        if let Ok(key_str) = api_key_val.to_str() {
            if let Some(key) = auth.validate_api_key(key_str) {
                // If they provided API key, we could generate a JWT or just mock claims
                let claims = crate::auth::Claims {
                    sub: key.user_id.clone(),
                    role: format!("{:?}", key.role),
                    exp: 0,
                };
                token_claims = Some(claims);
                _api_key_role = Some(key.role.clone());
            }
        }
    }

    if let Some(claims) = token_claims {
        let role_str = claims.role.clone();
        if let Some(role) = Role::from_str(&role_str) {
            if RoleEngine::can_access(&role, &path, &method) {
                req.extensions_mut().insert(claims);
                return Ok(next.run(req).await);
            } else {
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}
