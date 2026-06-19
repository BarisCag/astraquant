use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TenorBucket {
    Overnight,
    OneWeek,
    OneMonth,
    ThreeMonth,
    SixMonth,
    OneYear,
    TwoYear,
    FiveYear,
    TenYear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CHF,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RateSensitivity {
    pub tenor: TenorBucket,
    pub currency: Currency,
    pub modified_duration: f64,  // years
    pub convexity: f64,
    pub notional: i64,           // fixed-point 10^8
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HedgeRatio {
    pub instrument: String,
    pub ratio: f64,              // bounded [0.0, 1.0]
    pub notional: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ALMMismatchReport {
    pub tenor: TenorBucket,
    pub currency: Currency,
    pub asset_duration: f64,
    pub liability_duration: f64,
    pub duration_gap: f64,       // asset - liability
    pub notional_gap: i64,       // asset - liability notional
}
