use crate::types::{PortfolioSnapshot, Position};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PositionEngine {
    pub positions: BTreeMap<u64, BTreeMap<String, Position>>,
}

impl PositionEngine {
    pub fn new() -> Self {
        Self {
            positions: BTreeMap::new(),
        }
    }

    pub fn apply_fill(
        &mut self,
        trader_id: u64,
        symbol: &str,
        is_buy: bool,
        fill_quantity: u64,
        fill_price: i64,
    ) {
        let trader_positions = self.positions.entry(trader_id).or_default();
        let pos = trader_positions
            .entry(symbol.to_string())
            .or_insert(Position {
                trader_id,
                symbol: symbol.to_string(),
                net_quantity: 0,
                average_entry_price: 0,
                realized_pnl: 0,
                unrealized_pnl: 0,
                last_mark_price: fill_price,
            });

        let trade_qty = if is_buy {
            fill_quantity as i64
        } else {
            -(fill_quantity as i64)
        };

        if pos.net_quantity == 0 {
            pos.net_quantity = trade_qty;
            pos.average_entry_price = fill_price;
        } else if (pos.net_quantity > 0 && is_buy) || (pos.net_quantity < 0 && !is_buy) {
            let current_notional = pos.net_quantity.saturating_mul(pos.average_entry_price);
            let trade_notional = trade_qty.saturating_mul(fill_price);
            pos.net_quantity = pos.net_quantity.saturating_add(trade_qty);
            pos.average_entry_price =
                current_notional.saturating_add(trade_notional) / pos.net_quantity;
        } else {
            let closing_qty = if pos.net_quantity.abs() <= trade_qty.abs() {
                pos.net_quantity.abs()
            } else {
                trade_qty.abs()
            };

            let pnl_per_share = if pos.net_quantity > 0 {
                fill_price.saturating_sub(pos.average_entry_price)
            } else {
                pos.average_entry_price.saturating_sub(fill_price)
            };

            pos.realized_pnl = pos
                .realized_pnl
                .saturating_add(closing_qty.saturating_mul(pnl_per_share));

            let old_qty = pos.net_quantity;
            pos.net_quantity = pos.net_quantity.saturating_add(trade_qty);

            if pos.net_quantity == 0 {
                pos.average_entry_price = 0;
            } else if old_qty.signum() != pos.net_quantity.signum() {
                pos.average_entry_price = fill_price;
            }
        }

        pos.last_mark_price = fill_price;
    }

    pub fn update_mark_price(&mut self, symbol: &str, mark_price: i64) {
        for symbols in self.positions.values_mut() {
            if let Some(pos) = symbols.get_mut(symbol) {
                pos.last_mark_price = mark_price;
            }
        }
    }

    pub fn generate_snapshot(&mut self, trader_id: u64) -> PortfolioSnapshot {
        let mut snapshot = PortfolioSnapshot {
            gross_exposure: 0,
            net_exposure: 0,
            total_realized_pnl: 0,
            total_unrealized_pnl: 0,
            active_symbol_count: 0,
            inventory_concentration_metrics: BTreeMap::new(),
        };

        if let Some(symbols) = self.positions.get_mut(&trader_id) {
            for (symbol, pos) in symbols.iter_mut() {
                if pos.net_quantity > 0 {
                    pos.unrealized_pnl = pos.net_quantity.saturating_mul(
                        pos.last_mark_price.saturating_sub(pos.average_entry_price),
                    );
                } else if pos.net_quantity < 0 {
                    pos.unrealized_pnl = pos.net_quantity.abs().saturating_mul(
                        pos.average_entry_price.saturating_sub(pos.last_mark_price),
                    );
                } else {
                    pos.unrealized_pnl = 0;
                }

                let notional = pos.net_quantity.abs().saturating_mul(pos.last_mark_price);
                let signed_notional = pos.net_quantity.saturating_mul(pos.last_mark_price);

                snapshot.gross_exposure = snapshot.gross_exposure.saturating_add(notional);
                snapshot.net_exposure = snapshot.net_exposure.saturating_add(signed_notional);
                snapshot.total_realized_pnl =
                    snapshot.total_realized_pnl.saturating_add(pos.realized_pnl);
                snapshot.total_unrealized_pnl = snapshot
                    .total_unrealized_pnl
                    .saturating_add(pos.unrealized_pnl);

                if pos.net_quantity != 0 {
                    snapshot.active_symbol_count += 1;
                    snapshot
                        .inventory_concentration_metrics
                        .insert(symbol.clone(), signed_notional);
                }
            }
        }

        snapshot
    }
}
