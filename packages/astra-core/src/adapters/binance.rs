//! Binance depth JSON normalization using fixed-point parsing (no floats).

use crate::depth::DepthDelta;
use crate::types::{Price, Quantity, PRICE_SCALE, QUANTITY_SCALE};
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BinanceDepthUpdate {
    pub e: String,
    pub E: u64,
    pub s: String,
    pub U: u64,
    pub u: u64,
    pub b: Vec<[String; 2]>,
    pub a: Vec<[String; 2]>,
}

fn scale_decimal_digits(scale: i64) -> usize {
    let mut digits = 0usize;
    let mut temp = scale;
    while temp > 1 {
        digits += 1;
        temp /= 10;
    }
    digits
}

fn parse_scaled_i64(value: &str, scale: i64) -> Option<i64> {
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
            .saturating_mul(10)
            .saturating_add((ch as i64) - ('0' as i64));
        parsed_frac_len += 1;
    }
    for _ in parsed_frac_len..frac_digits {
        frac_part = frac_part.saturating_mul(10);
    }

    let mut result = whole_part.saturating_mul(scale).saturating_add(frac_part);
    if negative {
        result = -result;
    }
    Some(result)
}

fn parse_scaled_u64(value: &str, scale: u64) -> Option<u64> {
    parse_scaled_i64(value, scale as i64).and_then(|v| if v >= 0 { Some(v as u64) } else { None })
}

pub fn normalize_binance_depth(raw: &BinanceDepthUpdate) -> Vec<DepthDelta> {
    let mut deltas = Vec::new();

    for bid in &raw.b {
        if let (Some(price), Some(qty)) = (
            parse_scaled_i64(&bid[0], PRICE_SCALE),
            parse_scaled_u64(&bid[1], QUANTITY_SCALE),
        ) {
            deltas.push(DepthDelta {
                price: Price::new(price),
                quantity: Quantity::new(qty),
                is_bid: true,
            });
        }
    }

    for ask in &raw.a {
        if let (Some(price), Some(qty)) = (
            parse_scaled_i64(&ask[0], PRICE_SCALE),
            parse_scaled_u64(&ask[1], QUANTITY_SCALE),
        ) {
            deltas.push(DepthDelta {
                price: Price::new(price),
                quantity: Quantity::new(qty),
                is_bid: false,
            });
        }
    }

    deltas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_decimal_without_float() {
        assert_eq!(parse_scaled_i64("50.5", 10_000), Some(505_000));
        assert_eq!(parse_scaled_u64("1.25", 10_000), Some(12_500));
    }
}
