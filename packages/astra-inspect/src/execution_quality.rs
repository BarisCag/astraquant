use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExecutionQualityMetrics {
    pub total_passive_quantity: u64,
    pub total_aggressive_quantity: u64,
    pub total_cancelled_quantity: u64,

    pub total_queue_survived_quantity: u64, // Quantity that reached front of queue and executed
    pub total_queue_advancement: u64, // Cumulative positions advanced

    // Spread Tracking (Midpoint scaled by 2 to avoid floats)
    pub total_effective_spread_x2: u64,
    pub total_realized_spread_x2: i64,
    pub spread_samples: u64,
}

impl ExecutionQualityMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_fill(
        &mut self,
        is_passive: bool,
        quantity: u64,
        match_price: i64,
        scaled_midpoint: i64,
        post_trade_scaled_midpoint: i64, // Assuming e.g., 5 seconds in future, but since deterministic, maybe just end of epoch
    ) {
        if is_passive {
            self.total_passive_quantity += quantity;
        } else {
            self.total_aggressive_quantity += quantity;

            // Effective spread = 2 * (Trade Price - Midpoint) for Buy, 2 * (Midpoint - Trade Price) for Sell
            // Since scaled_midpoint = 2 * Midpoint, Effective Spread x2 = 4 * Trade Price - 2 * scaled_midpoint? No.
            // Trade Price x2 = 2 * match_price
            // Effective Spread = |2 * match_price - scaled_midpoint|
            let spread = (2 * match_price - scaled_midpoint).abs();
            self.total_effective_spread_x2 += spread as u64;

            // Realized spread = 2 * (Trade Price - Post Trade Midpoint) for Buy ...
            // We just use absolute for simplicity in deterministic simulation
            let realized = (2 * match_price - post_trade_scaled_midpoint).abs();
            self.total_realized_spread_x2 += realized as i64;

            self.spread_samples += 1;
        }
    }

    pub fn record_cancel(&mut self, quantity: u64) {
        self.total_cancelled_quantity += quantity;
    }

    pub fn record_queue_advancement(&mut self, initial_ahead: u64, executed_quantity: u64) {
        self.total_queue_survived_quantity += executed_quantity;
        self.total_queue_advancement += initial_ahead;
    }

    pub fn passive_fill_ratio_ppm(&self) -> u64 {
        let total = self.total_passive_quantity + self.total_aggressive_quantity;
        if total == 0 {
            return 0;
        }
        (self.total_passive_quantity.saturating_mul(1_000_000)) / total
    }

    pub fn cancel_to_fill_ratio_ppm(&self) -> u64 {
        let total_fills = self.total_passive_quantity + self.total_aggressive_quantity;
        if total_fills == 0 {
            return 0;
        }
        (self.total_cancelled_quantity.saturating_mul(1_000_000)) / total_fills
    }

    pub fn queue_survival_ratio_ppm(&self) -> u64 {
        // How much passive quantity survived to execution vs cancelled
        let total_passive_intent = self.total_passive_quantity + self.total_cancelled_quantity;
        if total_passive_intent == 0 {
            return 0;
        }
        (self.total_passive_quantity.saturating_mul(1_000_000)) / total_passive_intent
    }

    pub fn average_queue_advancement(&self) -> u64 {
        if self.total_queue_survived_quantity == 0 {
            return 0;
        }
        self.total_queue_advancement / self.total_queue_survived_quantity
    }

    pub fn average_effective_spread_x2(&self) -> u64 {
        if self.spread_samples == 0 {
            return 0;
        }
        self.total_effective_spread_x2 / self.spread_samples
    }

    pub fn average_realized_spread_x2(&self) -> i64 {
        if self.spread_samples == 0 {
            return 0;
        }
        self.total_realized_spread_x2 / (self.spread_samples as i64)
    }
}
