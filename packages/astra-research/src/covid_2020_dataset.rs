//! 2020 COVID Crash — Synthetic Crisis Dataset Generator
//!
//! Generates a realistic ~800-event sequence representing March 16, 2020.
//! All prices are fixed-point (PRICE_SCALE = 10_000).
//! 2750.00 = 27_500_000 internal units.
//!
//! Phases:
//!   [0–199]   Circuit breaker trigger — extreme drop 2750 -> 2190
//!   [200–399] Halt — market paused
//!   [400–599] Resume — high volatility, slightly up to 2250
//!   [600–799] Volatility — erratic trading settling at 2400

use astra_core::events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata};
use astra_core::marketdata::MarketTick;
use astra_core::serialization::serialize_canonical;
use astra_core::types::{Price, Quantity};

pub fn build_covid_crash_events() -> Vec<AstraEvent> {
    let mut events: Vec<AstraEvent> = Vec::with_capacity(800);
    let symbol = "SPX";

    // Price control points (in raw Price units: dollar * 10_000)
    let phase_prices: &[(u64, u64, i64, i64)] = &[
        (0,   199, 27_500_000, 21_900_000),  // Crash
        (200, 399, 21_900_000, 21_900_000),  // Halt
        (400, 599, 21_900_000, 22_500_000),  // Resume
        (600, 799, 22_500_000, 24_000_000),  // Volatility
    ];

    for &(seq_start, seq_end, start_price, end_price) in phase_prices {
        let count = (seq_end - seq_start + 1) as i64;
        let price_step = (end_price - start_price) / count.max(1);

        for i in seq_start..=seq_end {
            let seq = i + 1;
            
            // Inject some volatility
            let volatility = if i >= 600 {
                if i % 2 == 0 { 200_000 } else { -150_000 }
            } else {
                0
            };
            
            let price_raw = start_price + price_step * (i - seq_start) as i64 + volatility;

            let event_type = classify_event(i, seq_start, seq_end);

            let half_spread = if i >= 200 && i < 400 { 50_000 } else { 5_000 };
            let bid = Price::new(price_raw - half_spread);
            let ask = Price::new(price_raw + half_spread);

            let volume_raw: u64 = if i >= 200 && i < 400 {
                0 // Halted
            } else if i >= 400 && i < 600 {
                15_000 // Flood of orders on resume
            } else {
                3_000
            };

            let tick = MarketTick {
                symbol: symbol.to_string(),
                timestamp_ns: seq,
                bid_price: bid,
                ask_price: ask,
                bid_quantity: Quantity::new(volume_raw),
                ask_quantity: Quantity::new(volume_raw),
            };

            let payload = serialize_canonical(&tick).expect("tick serialization failed");

            events.push(AstraEvent::new(
                seq,
                seq,
                event_type,
                payload,
                PayloadMetadata::new(PayloadEncoding::Bincode, 1),
            ));
        }
    }

    events
}

fn classify_event(i: u64, phase_start: u64, _phase_end: u64) -> EventType {
    let pos = i - phase_start;

    match phase_start {
        0 => {
            if pos == 199 { EventType::CircuitBreakerTriggered }
            else { EventType::MarketTick }
        }
        200 => EventType::MarketTick, // Halted market ticks
        400 => {
            if pos == 0 { EventType::SystemRecovery } // Resume
            else { EventType::MarketTick }
        }
        _ => EventType::MarketTick,
    }
}
