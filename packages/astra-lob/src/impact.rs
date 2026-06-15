use crate::types::{LiquiditySide, TradeExecution};
use astra_core::types::Quantity;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImpactMetrics {
    pub levels_swept: u64,
    pub quantity_consumed: Quantity,
    pub sweep_cost_ppm: i64,
    pub adverse_price_distance: i64,
}

pub fn calculate_impact(executions: &[TradeExecution], aggressive_order_id: u64, arrival_best_price: i64) -> ImpactMetrics {
    let mut total_qty = 0;
    let mut total_cost = 0;
    let mut levels = BTreeMap::new();
    let mut worst_price = arrival_best_price;

    for exec in executions {
        if exec.aggressive_order_id == aggressive_order_id && exec.liquidity_side == LiquiditySide::Taker {
            let qty = exec.matched_quantity.0;
            let price = exec.match_price.0;
            
            total_qty += qty;
            total_cost += qty as i64 * (price - arrival_best_price).abs(); // absolute diff represents cost
            
            *levels.entry(price).or_insert(0) += qty;
            
            if (price - arrival_best_price).abs() > (worst_price - arrival_best_price).abs() {
                worst_price = price;
            }
        }
    }

    let sweep_cost_ppm = if total_qty > 0 {
        (total_cost * 1_000_000) / (total_qty as i64)
    } else {
        0
    };

    ImpactMetrics {
        levels_swept: levels.len() as u64,
        quantity_consumed: Quantity(total_qty),
        sweep_cost_ppm,
        adverse_price_distance: (worst_price - arrival_best_price).abs(),
    }
}
