use std::collections::VecDeque;
use crate::portfolio::PortfolioTracker;
use crate::types::PaperOrder;

#[derive(Clone, Copy, Debug)]
pub struct RiskLimits {
    pub max_notional_per_symbol: i64,
    pub max_drawdown_usd: i64,
    pub max_orders_per_second: usize,
}

pub struct RiskEngine {
    pub limits: RiskLimits,
    pub initial_nav: i64,
    pub high_watermark: i64,
    
    order_timestamps: VecDeque<u64>,
    pub kill_switch_triggered: bool,
}

impl RiskEngine {
    pub fn new(limits: RiskLimits, initial_nav: i64) -> Self {
        Self {
            limits,
            initial_nav,
            high_watermark: initial_nav,
            order_timestamps: VecDeque::new(),
            kill_switch_triggered: false,
        }
    }

    pub fn filter_order(
        &mut self,
        order: &PaperOrder,
        current_time_ns: u64,
        current_price: u64,
        portfolio: &PortfolioTracker,
        current_nav: i64,
    ) -> Result<(), String> {
        if self.kill_switch_triggered {
            return Err("Kill switch is active. Engine halted.".to_string());
        }

        self.high_watermark = std::cmp::max(self.high_watermark, current_nav);
        if self.high_watermark - current_nav > self.limits.max_drawdown_usd {
            self.kill_switch_triggered = true;
            return Err(format!("Max drawdown breached! NAV dropped to {}", current_nav));
        }

        // Clean up old timestamps (older than 1 second)
        let one_sec_ns = 1_000_000_000;
        while let Some(&ts) = self.order_timestamps.front() {
            if current_time_ns.saturating_sub(ts) > one_sec_ns {
                self.order_timestamps.pop_front();
            } else {
                break;
            }
        }

        if self.order_timestamps.len() >= self.limits.max_orders_per_second {
            self.kill_switch_triggered = true;
            return Err("Order rate limit breached! Kill switch activated.".to_string());
        }

        self.order_timestamps.push_back(current_time_ns);

        let _order_notional = ((order.quantity as i128 * current_price as i128) / 100_000_000) as i64;
        let current_position = portfolio.positions.get(&order.symbol).map(|p| p.quantity).unwrap_or(0);
        
        let mut new_quantity = current_position;
        match order.side {
            crate::types::Side::Buy => new_quantity += order.quantity as i64,
            crate::types::Side::Sell => new_quantity -= order.quantity as i64,
        }

        let projected_notional = ((new_quantity.abs() as i128 * current_price as i128) / 100_000_000) as i64;

        if projected_notional > self.limits.max_notional_per_symbol {
            self.kill_switch_triggered = true;
            return Err("Max notional breached! Kill switch activated.".to_string());
        }

        Ok(())
    }
}
