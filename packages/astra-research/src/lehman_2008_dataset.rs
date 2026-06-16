//! 2008 Lehman Collapse — Synthetic Crisis Dataset Generator
//!
//! Generates a realistic ~1200-event sequence representing September 15, 2008.
//! All prices are fixed-point (PRICE_SCALE = 10_000).
//! 1250.00 = 12_500_000 internal units.
//!
//! Phases:
//!   [0–299]     Credit freeze — normal-ish but widening spreads, price drops slowly
//!   [300–599]   Bank contagion — sharp drop, risk events
//!   [600–899]   Liquidity collapse — nadir reached (1050), margin calls
//!   [900–1199]  Fed intervention — partial recovery to 1100

use astra_core::events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata};
use astra_core::marketdata::MarketTick;
use astra_core::serialization::serialize_canonical;
use astra_core::types::{Price, Quantity};

pub fn build_lehman_collapse_events() -> Vec<AstraEvent> {
    let mut events: Vec<AstraEvent> = Vec::with_capacity(1200);
    let symbol = "SPX";

    // Price control points (in raw Price units: dollar * 10_000)
    let phase_prices: &[(u64, u64, i64, i64)] = &[
        //  (seq_start, seq_end, start_price_raw, end_price_raw)
        (0, 299, 12_500_000, 12_200_000),    // Credit freeze
        (300, 599, 12_200_000, 11_500_000),  // Bank contagion
        (600, 899, 11_500_000, 10_500_000),  // Liquidity collapse
        (900, 1199, 10_500_000, 11_000_000), // Fed intervention
    ];

    for &(seq_start, seq_end, start_price, end_price) in phase_prices {
        let count = (seq_end - seq_start + 1) as i64;
        let price_step = (end_price - start_price) / count.max(1);

        for i in seq_start..=seq_end {
            let seq = i + 1;
            let price_raw = start_price + price_step * (i - seq_start) as i64;

            let event_type = classify_event(i, seq_start, seq_end);

            let half_spread = if (600..900).contains(&i) {
                10_000
            } else {
                2_000
            };
            let bid = Price::new(price_raw - half_spread);
            let ask = Price::new(price_raw + half_spread);

            let volume_raw: u64 = if (600..900).contains(&i) {
                2_000 // Liquidity dried up
            } else if i >= 900 {
                10_000 // Fed injection
            } else {
                5_000
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
        0 => EventType::MarketTick,
        300 => {
            if pos.is_multiple_of(50) {
                EventType::InvariantViolationDetected
            }
            // Re-using as Contagion
            else {
                EventType::MarketTick
            }
        }
        600 => {
            if pos.is_multiple_of(40) {
                EventType::LiquidationExecuted
            } else if pos.is_multiple_of(20) {
                EventType::SettlementFailed
            } else {
                EventType::MarketTick
            }
        }
        900 => {
            if pos == 0 {
                EventType::LiquidityFacilityActivated
            } else if pos.is_multiple_of(50) {
                EventType::PolicyAction
            } else {
                EventType::MarketTick
            }
        }
        _ => EventType::MarketTick,
    }
}
