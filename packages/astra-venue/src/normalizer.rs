use astra_core::events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata};
use serde::{Deserialize, Serialize};

use crate::binance::RawTrade;

#[derive(Serialize, Deserialize, Debug)]
pub struct NormalizedTradePayload {
    pub symbol: String,
    pub price: u64,
    pub quantity: u64,
}

pub struct TradeNormalizer;

impl TradeNormalizer {
    pub fn normalize(trade: &RawTrade, sequence_id: u64) -> AstraEvent {
        let price = Self::parse_fixed_point(&trade.price_str);
        let quantity = Self::parse_fixed_point(&trade.quantity_str);
        
        let payload = NormalizedTradePayload {
            symbol: trade.symbol.clone(),
            price,
            quantity,
        };
        
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        
        AstraEvent::new(
            trade.timestamp_ms * 1_000_000,
            sequence_id,
            EventType::MarketTick,
            payload_bytes,
            PayloadMetadata::new(PayloadEncoding::Json, 1),
        )
    }

    fn parse_fixed_point(val: &str) -> u64 {
        let parts: Vec<&str> = val.split('.').collect();
        let integer_part: u64 = parts[0].parse().unwrap_or(0);
        let mut fractional_part: u64 = 0;
        
        if parts.len() > 1 {
            let frac_str = parts[1];
            // Take up to 8 digits
            let limit = std::cmp::min(frac_str.len(), 8);
            let parsed: u64 = frac_str[..limit].parse().unwrap_or(0);
            
            // Pad with zeros to reach 8 decimal places
            let pad = 8 - limit;
            fractional_part = parsed * 10u64.pow(pad as u32);
        }
        
        integer_part * 100_000_000 + fractional_part
    }
}
