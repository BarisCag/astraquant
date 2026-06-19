use std::collections::HashMap;
use crate::events::FillEvent;
use crate::types::Side;

pub struct Position {
    pub symbol: String,
    pub quantity: i64,
    pub entry_notional: i64,
}

pub struct PortfolioTracker {
    pub cash_balance: i64,
    pub realized_pnl: i64,
    pub positions: HashMap<String, Position>,
}

impl PortfolioTracker {
    pub fn new(initial_balance: i64) -> Self {
        Self {
            cash_balance: initial_balance,
            realized_pnl: 0,
            positions: HashMap::new(),
        }
    }

    pub fn update_from_fill(&mut self, fill: &FillEvent) {
        let qty = fill.fill_quantity as i64;
        let price = fill.fill_price as i64;
        let notional = ((qty as i128 * price as i128) / 100_000_000) as i64;

        let pos = self.positions.entry(fill.symbol.clone()).or_insert(Position {
            symbol: fill.symbol.clone(),
            quantity: 0,
            entry_notional: 0,
        });

        match fill.side {
            Side::Buy => {
                self.cash_balance -= notional;
                if pos.quantity < 0 {
                    let cover_qty = std::cmp::min(qty, -pos.quantity);
                    let avg_entry_price = pos.entry_notional / (-pos.quantity);
                    let pnl = ((avg_entry_price - price) as i128 * cover_qty as i128 / 100_000_000) as i64;
                    self.realized_pnl += pnl;
                    
                    pos.quantity += qty;
                    if pos.quantity == 0 {
                        pos.entry_notional = 0;
                    } else if pos.quantity < 0 {
                        pos.entry_notional = (-pos.quantity) * avg_entry_price;
                    } else {
                        pos.entry_notional = ((pos.quantity as i128 * price as i128) / 100_000_000) as i64;
                    }
                } else {
                    pos.quantity += qty;
                    pos.entry_notional += notional;
                }
            }
            Side::Sell => {
                self.cash_balance += notional;
                if pos.quantity > 0 {
                    let sell_qty = std::cmp::min(qty, pos.quantity);
                    let avg_entry_price = pos.entry_notional / pos.quantity;
                    let pnl = ((price - avg_entry_price) as i128 * sell_qty as i128 / 100_000_000) as i64;
                    self.realized_pnl += pnl;
                    
                    pos.quantity -= qty;
                    if pos.quantity == 0 {
                        pos.entry_notional = 0;
                    } else if pos.quantity > 0 {
                        pos.entry_notional = pos.quantity * avg_entry_price;
                    } else {
                        pos.entry_notional = (((-pos.quantity) as i128 * price as i128) / 100_000_000) as i64;
                    }
                } else {
                    pos.quantity -= qty;
                    pos.entry_notional += notional;
                }
            }
        }
    }

    pub fn total_equity(&self, current_prices: &HashMap<String, u64>) -> i64 {
        self.cash_balance + self.unrealized_pnl(current_prices)
    }

    pub fn unrealized_pnl(&self, current_prices: &HashMap<String, u64>) -> i64 {
        let mut u_pnl = 0;
        for pos in self.positions.values() {
            if pos.quantity == 0 {
                continue;
            }
            if let Some(&price) = current_prices.get(&pos.symbol) {
                let current_notional = ((pos.quantity.abs() as i128 * price as i128) / 100_000_000) as i64;
                if pos.quantity > 0 {
                    u_pnl += current_notional - pos.entry_notional;
                } else {
                    u_pnl += pos.entry_notional - current_notional;
                }
            }
        }
        u_pnl
    }
}
