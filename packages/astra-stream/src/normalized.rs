//! Deterministic normalized market event types and fixed-point parsing.
//!
//! All pricing and quantity data uses integer fixed-point representation
//! at 1e8 scale. No floating-point arithmetic is used anywhere in this module.
//! Conversions between `STREAM_PRICE_SCALE` (1e8) and `astra_core::types::PRICE_SCALE`
//! (1e4) must be explicit and deterministic.

use astra_core::hashing::{hash_bytes, DeterministicState};
use astra_core::serialization::{deserialize_canonical, serialize_canonical, SerializationError};
use serde::{Deserialize, Serialize};

/// Fixed-point scale for stream-side price normalization (8 decimal places).
/// This is distinct from `astra_core::types::PRICE_SCALE` (4 decimal places).
/// All conversions between scales must be explicit.
pub const STREAM_PRICE_SCALE: i64 = 100_000_000;

/// Fixed-point scale for stream-side quantity normalization (8 decimal places).
/// This is distinct from `astra_core::types::QUANTITY_SCALE` (4 decimal places).
pub const STREAM_QUANTITY_SCALE: u64 = 100_000_000;

/// Fixed-size symbol identifier. ASCII uppercase, zero-padded to 16 bytes.
/// Deterministic serialization: no heap allocation, fixed wire size.
pub type Symbol = [u8; 16];

/// Convert a string symbol to a fixed-size [`Symbol`].
/// Input is uppercased and truncated to 16 bytes.
pub fn symbol_from_str(s: &str) -> Symbol {
    let mut sym = [0u8; 16];
    let upper = s.to_ascii_uppercase();
    let bytes = upper.as_bytes();
    let len = bytes.len().min(16);
    sym[..len].copy_from_slice(&bytes[..len]);
    sym
}

/// Convert a fixed-size [`Symbol`] back to a [`String`].
pub fn symbol_to_string(sym: &Symbol) -> String {
    let end = sym.iter().position(|&b| b == 0).unwrap_or(16);
    String::from_utf8_lossy(&sym[..end]).into_owned()
}

/// Canonical deterministic market event from an exchange trade stream.
///
/// All fields except `receive_timestamp_ns` participate in deterministic hashing.
/// `receive_timestamp_ns` is wall-clock ingest time, preserved for latency analysis
/// but excluded from replay verification.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedMarketEvent {
    /// Exchange-reported trade time in microseconds since Unix epoch.
    /// Binance reports milliseconds; we store as microseconds for uniform precision.
    pub exchange_timestamp_us: u64,
    /// Wall-clock ingest time in nanoseconds. **NON-DETERMINISTIC**.
    /// Excluded from [`DeterministicState::state_hash`] and replay verification.
    pub receive_timestamp_ns: u64,
    /// Fixed-size symbol identifier (e.g. "BTCUSDT" → `[B,T,C,U,S,D,T,0,0,...]`).
    pub symbol: Symbol,
    /// Trade price in fixed-point integer, scale [`STREAM_PRICE_SCALE`] (1e8).
    pub price: i64,
    /// Trade quantity in fixed-point integer, scale [`STREAM_QUANTITY_SCALE`] (1e8).
    pub quantity: u64,
    /// True if the buyer is the market maker (i.e. the trade was a sell).
    pub is_buyer_maker: bool,
    /// Exchange-assigned trade ID.
    pub trade_id: u64,
}

impl DeterministicState for NormalizedMarketEvent {
    /// Compute a deterministic BLAKE3 hash of this event.
    /// `receive_timestamp_ns` is deliberately excluded.
    fn state_hash(&self) -> [u8; 32] {
        let mut data = Vec::with_capacity(57);
        data.extend_from_slice(&self.exchange_timestamp_us.to_le_bytes());
        // NOTE: receive_timestamp_ns deliberately excluded from deterministic hash
        data.extend_from_slice(&self.symbol);
        data.extend_from_slice(&self.price.to_le_bytes());
        data.extend_from_slice(&self.quantity.to_le_bytes());
        data.push(self.is_buyer_maker as u8);
        data.extend_from_slice(&self.trade_id.to_le_bytes());
        hash_bytes(&data)
    }
}

impl NormalizedMarketEvent {
    /// Serialize this event into a journal-ready payload via canonical bincode.
    pub fn to_journal_payload(&self) -> Result<Vec<u8>, SerializationError> {
        serialize_canonical(self)
    }

    /// Deserialize a [`NormalizedMarketEvent`] from an `AstraEvent` payload.
    pub fn from_payload(payload: &[u8]) -> Result<Self, SerializationError> {
        deserialize_canonical(payload)
    }
}

// ---------------------------------------------------------------------------
// Integer-only decimal string parsing
// ---------------------------------------------------------------------------

/// Count the number of decimal digits in a power-of-10 scale factor.
fn scale_decimal_digits(scale: i64) -> usize {
    let mut digits = 0usize;
    let mut temp = scale;
    while temp > 1 {
        digits += 1;
        temp /= 10;
    }
    digits
}

/// Parse a decimal string into a fixed-point `i64` at the given scale.
///
/// Uses pure integer arithmetic — no floating-point operations at any point.
/// Returns `None` on invalid input or arithmetic overflow.
///
/// # Examples
/// ```
/// use astra_stream::normalized::parse_decimal_fixed;
/// assert_eq!(parse_decimal_fixed("29345.67", 100_000_000), Some(2_934_567_000_000));
/// assert_eq!(parse_decimal_fixed("0.00100000", 100_000_000), Some(100_000));
/// assert_eq!(parse_decimal_fixed("100", 100_000_000), Some(10_000_000_000));
/// ```
pub fn parse_decimal_fixed(value: &str, scale: i64) -> Option<i64> {
    let negative = value.starts_with('-');
    let digits = value.trim_start_matches('-');
    let (whole, frac) = match digits.split_once('.') {
        Some((w, f)) => (w, f),
        None => (digits, ""),
    };
    if whole.is_empty() && frac.is_empty() {
        return None;
    }
    let whole_part: i64 = if whole.is_empty() {
        0
    } else {
        whole.parse().ok()?
    };

    let frac_digits = scale_decimal_digits(scale);
    let mut frac_part: i64 = 0;
    let mut parsed_frac_len = 0usize;
    for ch in frac.chars().take(frac_digits) {
        if !ch.is_ascii_digit() {
            return None;
        }
        frac_part = frac_part
            .checked_mul(10)?
            .checked_add((ch as i64) - ('0' as i64))?;
        parsed_frac_len += 1;
    }
    for _ in parsed_frac_len..frac_digits {
        frac_part = frac_part.checked_mul(10)?;
    }

    let mut result = whole_part.checked_mul(scale)?.checked_add(frac_part)?;
    if negative {
        result = -result;
    }
    Some(result)
}

/// Parse a decimal string into a fixed-point `u64`. Returns `None` if negative.
pub fn parse_decimal_fixed_u64(value: &str, scale: u64) -> Option<u64> {
    parse_decimal_fixed(value, scale as i64)
        .and_then(|v| if v >= 0 { Some(v as u64) } else { None })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard_price() {
        assert_eq!(
            parse_decimal_fixed("29345.67", STREAM_PRICE_SCALE),
            Some(2_934_567_000_000)
        );
    }

    #[test]
    fn parse_binance_full_precision() {
        // Binance sends 8 decimal places: "29345.67000000"
        assert_eq!(
            parse_decimal_fixed("29345.67000000", STREAM_PRICE_SCALE),
            Some(2_934_567_000_000)
        );
    }

    #[test]
    fn parse_small_quantity() {
        assert_eq!(
            parse_decimal_fixed_u64("0.00100000", STREAM_QUANTITY_SCALE),
            Some(100_000)
        );
    }

    #[test]
    fn parse_whole_number() {
        assert_eq!(
            parse_decimal_fixed("100", STREAM_PRICE_SCALE),
            Some(10_000_000_000)
        );
    }

    #[test]
    fn parse_zero() {
        assert_eq!(parse_decimal_fixed("0", STREAM_PRICE_SCALE), Some(0));
        assert_eq!(
            parse_decimal_fixed("0.00000000", STREAM_PRICE_SCALE),
            Some(0)
        );
    }

    #[test]
    fn parse_negative() {
        assert_eq!(
            parse_decimal_fixed("-50.5", STREAM_PRICE_SCALE),
            Some(-5_050_000_000)
        );
    }

    #[test]
    fn parse_negative_u64_returns_none() {
        assert_eq!(parse_decimal_fixed_u64("-1.0", STREAM_QUANTITY_SCALE), None);
    }

    #[test]
    fn parse_trailing_decimals_truncated_to_scale() {
        // 9 decimal places but scale only has 8 → 9th digit silently truncated
        assert_eq!(
            parse_decimal_fixed("0.123456789", STREAM_PRICE_SCALE),
            Some(12_345_678)
        );
    }

    #[test]
    fn parse_empty_returns_none() {
        assert_eq!(parse_decimal_fixed("", STREAM_PRICE_SCALE), None);
    }

    #[test]
    fn parse_invalid_returns_none() {
        assert_eq!(parse_decimal_fixed("abc", STREAM_PRICE_SCALE), None);
        assert_eq!(parse_decimal_fixed("12.3x", STREAM_PRICE_SCALE), None);
    }

    #[test]
    fn parse_leading_dot() {
        assert_eq!(
            parse_decimal_fixed(".5", STREAM_PRICE_SCALE),
            Some(50_000_000)
        );
    }

    #[test]
    fn symbol_roundtrip() {
        let sym = symbol_from_str("BTCUSDT");
        assert_eq!(symbol_to_string(&sym), "BTCUSDT");
    }

    #[test]
    fn symbol_uppercase() {
        let sym = symbol_from_str("btcusdt");
        assert_eq!(symbol_to_string(&sym), "BTCUSDT");
    }

    #[test]
    fn symbol_truncation() {
        let sym = symbol_from_str("VERYLONGSYMBOLNAME123");
        assert_eq!(symbol_to_string(&sym).len(), 16);
    }

    #[test]
    fn deterministic_hash_excludes_receive_timestamp() {
        let event1 = NormalizedMarketEvent {
            exchange_timestamp_us: 1_000_000,
            receive_timestamp_ns: 999,
            symbol: symbol_from_str("BTCUSDT"),
            price: 2_934_567_000_000,
            quantity: 100_000,
            is_buyer_maker: true,
            trade_id: 42,
        };
        let event2 = NormalizedMarketEvent {
            receive_timestamp_ns: 12_345_678, // different receive time
            ..event1.clone()
        };
        assert_eq!(event1.state_hash(), event2.state_hash());
    }

    #[test]
    fn deterministic_hash_sensitive_to_price() {
        let event1 = NormalizedMarketEvent {
            exchange_timestamp_us: 1_000_000,
            receive_timestamp_ns: 0,
            symbol: symbol_from_str("BTCUSDT"),
            price: 100_000_000,
            quantity: 100_000,
            is_buyer_maker: false,
            trade_id: 1,
        };
        let event2 = NormalizedMarketEvent {
            price: 200_000_000, // different price
            ..event1.clone()
        };
        assert_ne!(event1.state_hash(), event2.state_hash());
    }

    #[test]
    fn serialization_roundtrip() {
        let event = NormalizedMarketEvent {
            exchange_timestamp_us: 1_625_000_000_000_000,
            receive_timestamp_ns: 1_625_000_000_000_000_000,
            symbol: symbol_from_str("ETHUSDT"),
            price: 200_000_000_000,
            quantity: 1_500_000_000,
            is_buyer_maker: false,
            trade_id: 98765,
        };
        let bytes = event.to_journal_payload().unwrap();
        let decoded = NormalizedMarketEvent::from_payload(&bytes).unwrap();
        assert_eq!(event, decoded);
    }
}
