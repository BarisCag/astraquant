use astra_core::depth::DepthDelta;
use astra_core::events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata};
use astra_core::types::{Price, Quantity};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct BinanceLiveGateway;

impl BinanceLiveGateway {
    /// Non-deterministic ingest logic. Converts raw WSS payloads into canonical AstraEvents.
    pub fn ingest_trade_payload(
        seq_id: u64,
        raw_price: f64,
        raw_qty: f64,
        is_bid: bool,
    ) -> AstraEvent {
        // Absolute Wall-clock timestamp injection
        let timestamp_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        // Canonicalization: Float to Fixed-Point mapping (e.g. 8 decimals)
        let price = Price::new((raw_price * 100_000_000.0) as i64);
        let quantity = Quantity::new((raw_qty * 100_000_000.0) as u64);

        let delta = DepthDelta {
            price,
            quantity,
            is_bid,
        };

        let payload = astra_core::serialization::serialize_canonical(&delta).unwrap();

        AstraEvent::new(
            timestamp_ns,
            seq_id,
            EventType::MarketTick,
            payload,
            PayloadMetadata::new(PayloadEncoding::Bincode, 1),
        )
    }
}
