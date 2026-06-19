use crate::events::FillEvent;
use crate::prng::DeterministicPrng;
use crate::types::{MarketSnapshot, OrderType, PaperOrder};

#[derive(Clone)]
pub enum FillModel {
    Naive,
    Slippage { n_trades: usize, slippage_bps: u64 },
    OrderBookImpact { impact_decay: f64 },
}

pub struct PaperExecutionEngine {
    pub fill_model: FillModel,
}

impl PaperExecutionEngine {
    pub fn new(fill_model: FillModel) -> Self {
        Self { fill_model }
    }

    pub fn try_fill(
        &mut self,
        order: &PaperOrder,
        snapshot: &MarketSnapshot,
        prng: &mut DeterministicPrng,
        prng_nonce: u64,
    ) -> Option<FillEvent> {
        match self.fill_model {
            FillModel::Naive => {
                let fill_price = match order.order_type {
                    OrderType::Market => snapshot.last_price,
                    OrderType::Limit(limit) => match order.side {
                        crate::types::Side::Buy => {
                            if snapshot.last_price <= limit {
                                snapshot.last_price
                            } else {
                                return None;
                            }
                        }
                        crate::types::Side::Sell => {
                            if snapshot.last_price >= limit {
                                snapshot.last_price
                            } else {
                                return None;
                            }
                        }
                    },
                };

                Some(FillEvent {
                    symbol: order.symbol.clone(),
                    side: order.side,
                    fill_price,
                    fill_quantity: order.quantity,
                    timestamp_ns: snapshot.timestamp_ns,
                    model_used: "Naive".to_string(),
                    prng_nonce_at_fill: prng_nonce,
                })
            }
            FillModel::Slippage { slippage_bps, .. } => {
                let slip_ratio = prng.next_f64();

                let max_slip = (snapshot.last_price * slippage_bps) / 10000;
                let actual_slip = (max_slip as f64 * slip_ratio) as u64;

                let fill_price = match order.order_type {
                    OrderType::Market => match order.side {
                        crate::types::Side::Buy => snapshot.last_price.saturating_add(actual_slip),
                        crate::types::Side::Sell => snapshot.last_price.saturating_sub(actual_slip),
                    },
                    OrderType::Limit(limit) => match order.side {
                        crate::types::Side::Buy => {
                            if snapshot.last_price <= limit {
                                snapshot.last_price.saturating_add(actual_slip).min(limit)
                            } else {
                                return None;
                            }
                        }
                        crate::types::Side::Sell => {
                            if snapshot.last_price >= limit {
                                snapshot.last_price.saturating_sub(actual_slip).max(limit)
                            } else {
                                return None;
                            }
                        }
                    },
                };

                Some(FillEvent {
                    symbol: order.symbol.clone(),
                    side: order.side,
                    fill_price,
                    fill_quantity: order.quantity,
                    timestamp_ns: snapshot.timestamp_ns,
                    model_used: "Slippage".to_string(),
                    prng_nonce_at_fill: prng_nonce,
                })
            }
            FillModel::OrderBookImpact { impact_decay } => {
                let impact_ratio = prng.next_f64() * impact_decay;
                
                // Assume k * sqrt(Q) where k is 10 bps
                let base_impact = (snapshot.last_price * 10) / 10000;
                let q_factor = (order.quantity as f64 / 100_000_000.0).sqrt();
                let actual_impact = (base_impact as f64 * impact_ratio * q_factor) as u64;

                let fill_price = match order.side {
                    crate::types::Side::Buy => snapshot.last_price.saturating_add(actual_impact),
                    crate::types::Side::Sell => snapshot.last_price.saturating_sub(actual_impact),
                };

                Some(FillEvent {
                    symbol: order.symbol.clone(),
                    side: order.side,
                    fill_price,
                    fill_quantity: order.quantity,
                    timestamp_ns: snapshot.timestamp_ns,
                    model_used: "OrderBookImpact".to_string(),
                    prng_nonce_at_fill: prng_nonce,
                })
            }
        }
    }
}
