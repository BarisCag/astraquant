//! 2010 Flash Crash — Synthetic Crisis Dataset Generator
//!
//! Generates a realistic ~1000-event sequence representing May 6, 2010 14:32–15:00 ET.
//! All prices are fixed-point (PRICE_SCALE = 10_000).
//! 1145.00 = 11_450_000 internal units.
//!
//! Phases:
//!   [0–199]   Normal trading — tight spreads, routine order flow
//!   [200–399] Liquidity withdrawal — spreads widen, volume drops
//!   [400–599] Cascade — price collapses from 1145 → 1056, margin calls trigger
//!   [600–799] Partial recovery — circuit breaker effect, price recovers to ~1130
//!   [800–999] Stabilisation — price anchors, volume returns

use astra_core::events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata};
use astra_core::marketdata::MarketTick;
use astra_core::serialization::serialize_canonical;
use astra_core::types::{Price, Quantity};

// PRICE_SCALE = 10_000.  price_pts(1145.00) = 1145 * 10_000 = 11_450_000
#[allow(dead_code)]
const fn price_pts(dollars_x100: i64) -> i64 {
    dollars_x100 * 100 // dollars_x100 is price * 100, so * 100 gives * 10_000
}

pub fn build_flash_crash_events() -> Vec<AstraEvent> {
    let mut events: Vec<AstraEvent> = Vec::with_capacity(1000);
    let symbol = "ES";

    // Price control points (in raw Price units: dollar * 10_000)
    // 1145.00 → 11_450_000
    let phase_prices: &[(u64, u64, i64, i64)] = &[
        //  (seq_start, seq_end, start_price_raw, end_price_raw)
        (0, 199, 11_450_000, 11_420_000), // Normal: slight drift down
        (200, 399, 11_420_000, 11_250_000), // Withdrawal: accelerating decline
        (400, 599, 11_250_000, 10_560_000), // Cascade: -7.7% nadir
        (600, 799, 10_560_000, 11_300_000), // Recovery: sharp rebound
        (800, 999, 11_300_000, 11_300_000), // Stabilisation
    ];

    for &(seq_start, seq_end, start_price, end_price) in phase_prices {
        let count = (seq_end - seq_start + 1) as i64;
        let price_step = (end_price - start_price) / count.max(1);

        for i in seq_start..=seq_end {
            let seq = i + 1; // 1-indexed sequence IDs
            let price_raw = start_price + price_step * (i - seq_start) as i64;

            // Determine event type based on phase and position within phase
            let event_type = classify_event(i, seq_start, seq_end);

            // Bid/ask spread widens during cascade
            let half_spread = if (400..600).contains(&i) { 5_000 } else { 500 };
            let bid = Price::new(price_raw - half_spread);
            let ask = Price::new(price_raw + half_spread);

            // Volume spikes during cascade and recovery
            let volume_raw: u64 = if (400..600).contains(&i) {
                5_000 + (i - 400) * 100 // rising cascade volume
            } else if (600..700).contains(&i) {
                8_000 // peak recovery volume
            } else {
                1_000 + (i % 20) * 50 // normal variance
            };

            let tick = MarketTick {
                symbol: symbol.to_string(),
                timestamp_ns: seq, // logical sequence, not wall-clock
                bid_price: bid,
                ask_price: ask,
                bid_quantity: Quantity::new(volume_raw),
                ask_quantity: Quantity::new(volume_raw),
            };

            let payload = serialize_canonical(&tick).expect("tick serialization failed");

            events.push(AstraEvent::new(
                seq, // timestamp_ns = logical seq
                seq, // sequence_id
                event_type,
                payload,
                PayloadMetadata::new(PayloadEncoding::Bincode, 1),
            ));
        }
    }

    events
}

fn classify_event(i: u64, phase_start: u64, phase_end: u64) -> EventType {
    let _phase_len = phase_end - phase_start;
    let pos = i - phase_start;

    match phase_start {
        0 => EventType::MarketTick, // Normal: all market ticks
        200 => {
            // Withdrawal: mostly ticks, but inject limit order cancellations every 20
            if pos % 20 == 0 {
                EventType::LimitOrderCancelled
            } else {
                EventType::MarketTick
            }
        }
        400 => {
            // Cascade: margin calls and risk triggers mixed with ticks
            if pos % 30 == 0 {
                EventType::RiskThresholdTriggered
            } else if pos % 15 == 0 {
                EventType::MarginCallIssued
            } else {
                EventType::MarketTick
            }
        }
        600 => {
            // Recovery: circuit breaker at position 0, then stabilising ticks
            if pos == 0 {
                EventType::CircuitBreakerTriggered
            } else if pos % 50 == 0 {
                EventType::RegulatoryIntervention
            } else {
                EventType::MarketTick
            }
        }
        _ => EventType::MarketTick, // Stabilisation
    }
}
