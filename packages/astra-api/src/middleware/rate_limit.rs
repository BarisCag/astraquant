use axum::{
    extract::Request,
    http::{StatusCode, HeaderValue, header::RETRY_AFTER},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use crate::auth::Claims;
use crate::config::Config;

pub struct RateLimiter {
    requests_per_minute: u64,
    burst: u32,
    _window: std::time::Duration,
    state: RwLock<HashMap<String, Vec<u64>>>, // user_id -> timestamps
}

impl RateLimiter {
    pub fn new(config: &Config) -> Self {
        Self {
            requests_per_minute: config.rate_limit.requests_per_minute,
            burst: config.rate_limit.burst,
            _window: std::time::Duration::from_secs(60),
            state: RwLock::new(HashMap::new()),
        }
    }

    pub async fn check(&self, user_id: &str) -> Result<(), u64> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let window_start = now.saturating_sub(60);

        let mut state = self.state.write().await;
        let timestamps = state.entry(user_id.to_string()).or_insert_with(Vec::new);

        // Remove old requests
        timestamps.retain(|&t| t >= window_start);

        // Check burst
        let recent_burst_start = now.saturating_sub(1);
        let recent_burst_count = timestamps.iter().filter(|&&t| t >= recent_burst_start).count();
        
        if recent_burst_count >= self.burst as usize {
            return Err(1); // Retry after 1s for burst limit
        }

        // Check RPM
        if timestamps.len() >= self.requests_per_minute as usize {
            let oldest = timestamps.first().unwrap_or(&now);
            let retry_after = 60 - (now - oldest);
            return Err(retry_after.max(1));
        }

        timestamps.push(now);
        Ok(())
    }
}

pub async fn rate_limit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // Requires Claims to be injected by Auth middleware first
    let claims = req.extensions().get::<Claims>();
    
    // For health endpoint or unauthenticated paths, we might not have claims.
    // If we want to strictly rate limit per API key, we should do it after auth.
    // Let's assume unauthenticated gets global rate limit or bypass (for health).
    if let Some(claims) = claims {
        // Find RateLimiter in extensions
        if let Some(limiter) = req.extensions().get::<Arc<RateLimiter>>() {
            if let Err(retry_after) = limiter.check(&claims.sub).await {
                let mut res = Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .body(axum::body::Body::empty())
                    .unwrap();
                res.headers_mut().insert(
                    RETRY_AFTER,
                    HeaderValue::from_str(&retry_after.to_string()).unwrap(),
                );
                return Ok(res);
            }
        }
    }

    Ok(next.run(req).await)
}
