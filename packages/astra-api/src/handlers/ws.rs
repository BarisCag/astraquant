use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use serde::{Deserialize, Serialize};

use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

use crate::auth::AuthManager;

#[derive(Deserialize)]
pub struct WsQuery {
    pub token: String,
}

#[derive(Deserialize)]
pub struct SubscribeMsg {
    pub subscribe: Vec<String>,
}

#[derive(Serialize)]
pub struct MockEvent {
    pub event_type: String,
    pub timestamp_ns: u64,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(auth): State<Arc<AuthManager>>,
) -> axum::response::Result<Response, axum::http::StatusCode> {
    if auth.validate_jwt(&query.token).is_none() {
        return Err(axum::http::StatusCode::UNAUTHORIZED);
    }
    
    Ok(ws.on_upgrade(handle_socket))
}

async fn handle_socket(mut socket: WebSocket) {
    let (tx, mut rx) = mpsc::channel::<MockEvent>(1000); // Drop slow clients if 1000 messages buffered

    // Wait for subscription message
    let mut subscriptions = Vec::new();
    if let Some(Ok(Message::Text(text))) = socket.recv().await {
        if let Ok(sub) = serde_json::from_str::<SubscribeMsg>(&text) {
            subscriptions = sub.subscribe;
        }
    }

    if subscriptions.is_empty() {
        let _ = socket.close().await;
        return;
    }

    let mut ticker = interval(Duration::from_secs(2));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64;
                    
                for sub in &subscriptions {
                    let event = MockEvent {
                        event_type: sub.clone(),
                        timestamp_ns: now,
                    };
                    
                    if tx.try_send(event).is_err() {
                        // Slow client (buffer full) or receiver dropped
                        let _ = socket.close().await;
                        return;
                    }
                }
            }
            Some(event) = rx.recv() => {
                let text = serde_json::to_string(&event).unwrap();
                if socket.send(Message::Text(text)).await.is_err() {
                    break;
                }
            }
            msg = socket.recv() => {
                if let Some(Ok(Message::Close(_))) | None = msg {
                    break;
                }
            }
        }
    }
}
