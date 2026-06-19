use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;

use crate::demo::DemoMode;
use crate::handlers::ApiResponse;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UsersResponse {
    pub users: Vec<User>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KillswitchResponse {
    pub triggered: bool,
    pub timestamp_ns: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RotateKeyResponse {
    pub new_key: String,
    pub expires_at: u64,
}

pub async fn get_users(
    State(demo): State<Arc<DemoMode>>,
) -> Json<ApiResponse<UsersResponse>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

    let res = UsersResponse {
        users: vec![
            User {
                id: "user_1".to_string(),
                role: "Trader".to_string(),
            },
            User {
                id: "user_2".to_string(),
                role: "Admin".to_string(),
            },
        ],
    };

    Json(ApiResponse {
        data: res,
        timestamp_ns: now,
        mode: if demo.is_demo() { "demo".to_string() } else { "live".to_string() },
    })
}

pub async fn killswitch(
    State(demo): State<Arc<DemoMode>>,
) -> Json<ApiResponse<KillswitchResponse>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

    let res = KillswitchResponse {
        triggered: true,
        timestamp_ns: now,
    };

    Json(ApiResponse {
        data: res,
        timestamp_ns: now,
        mode: if demo.is_demo() { "demo".to_string() } else { "live".to_string() },
    })
}

pub async fn rotate_api_key(
    State(demo): State<Arc<DemoMode>>,
) -> Json<ApiResponse<RotateKeyResponse>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
    let expires = (SystemTime::now() + std::time::Duration::from_secs(86400))
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let res = RotateKeyResponse {
        new_key: "NEW_API_KEY_MOCK".to_string(),
        expires_at: expires,
    };

    Json(ApiResponse {
        data: res,
        timestamp_ns: now,
        mode: if demo.is_demo() { "demo".to_string() } else { "live".to_string() },
    })
}
