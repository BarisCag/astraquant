use futures_util::StreamExt;
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};

#[derive(Debug, Deserialize)]
pub struct RawTrade {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "p")]
    pub price_str: String,
    #[serde(rename = "q")]
    pub quantity_str: String,
    #[serde(rename = "T")]
    pub timestamp_ms: u64,
}

pub struct BinanceFeed {
    pub symbol: String,
    pub url: String,
}

impl BinanceFeed {
    pub fn new(symbol: &str) -> Self {
        let url = format!("wss://stream.binance.com:9443/ws/{}@trade", symbol.to_lowercase());
        Self {
            symbol: symbol.to_string(),
            url,
        }
    }

    pub async fn connect(&self) -> Result<BinanceStream, Box<dyn std::error::Error>> {
        let (ws_stream, _) = connect_async(&self.url).await?;
        Ok(BinanceStream { inner: ws_stream })
    }
}

pub struct BinanceStream {
    inner: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl BinanceStream {
    pub async fn next_trade(&mut self) -> Option<RawTrade> {
        while let Some(msg_result) = self.inner.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    if let Ok(trade) = serde_json::from_str::<RawTrade>(&text) {
                        return Some(trade);
                    }
                }
                Ok(Message::Close(_)) => return None,
                _ => {}
            }
        }
        None
    }
}
