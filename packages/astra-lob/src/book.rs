use crate::invariants::{IntegrityViolation, InvariantReport};
use crate::types::{LiquiditySide, Order, OrderEvent, OrderSide, OrderType, TradeExecution};
use astra_core::types::{Price, Quantity};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PriceLevel {
    pub orders: VecDeque<Order>,
}

impl PriceLevel {
    pub fn new() -> Self {
        Self {
            orders: VecDeque::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LimitOrderBook {
    pub symbol: String,
    // lowest price first, so we iterate forward for asks
    pub asks: BTreeMap<Price, PriceLevel>,
    // lowest price first, so we iterate .rev() for bids
    pub bids: BTreeMap<Price, PriceLevel>,
    // O(log N) lookup for cancel/modify
    pub orders: BTreeMap<u64, (OrderSide, Price)>,
}

impl LimitOrderBook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            asks: BTreeMap::new(),
            bids: BTreeMap::new(),
            orders: BTreeMap::new(),
        }
    }

    pub fn validate_invariants(&self) -> InvariantReport {
        let mut report = InvariantReport::new();

        // best_bid <= best_ask
        if let Some((&best_bid, _)) = self.bids.iter().next_back() {
            if let Some((&best_ask, _)) = self.asks.iter().next() {
                if best_bid >= best_ask {
                    report.violations.push(IntegrityViolation::CrossedBook);
                }
            }
        }

        let mut seen_orders = BTreeSet::new();

        // Check bids
        for (&price, level) in self.bids.iter() {
            for order in level.orders.iter() {
                if !seen_orders.insert(order.order_id) {
                    report
                        .violations
                        .push(IntegrityViolation::InvalidQueueOrdering {
                            order_id: order.order_id,
                        });
                }
                // Removed negative quantity check as Quantity is u64
                if order.remaining_quantity.0 == 0 {
                    report
                        .violations
                        .push(IntegrityViolation::ZeroQuantityOrder {
                            order_id: order.order_id,
                        });
                }
                if let Some(&(side, p)) = self.orders.get(&order.order_id) {
                    if side != OrderSide::Bid || p != price {
                        report
                            .violations
                            .push(IntegrityViolation::InvalidOrderIndex {
                                order_id: order.order_id,
                            });
                    }
                } else {
                    report
                        .violations
                        .push(IntegrityViolation::InvalidOrderIndex {
                            order_id: order.order_id,
                        });
                }
            }
        }

        // Check asks
        for (&price, level) in self.asks.iter() {
            for order in level.orders.iter() {
                if !seen_orders.insert(order.order_id) {
                    report
                        .violations
                        .push(IntegrityViolation::InvalidQueueOrdering {
                            order_id: order.order_id,
                        });
                }
                // Removed negative quantity check as Quantity is u64
                if order.remaining_quantity.0 == 0 {
                    report
                        .violations
                        .push(IntegrityViolation::ZeroQuantityOrder {
                            order_id: order.order_id,
                        });
                }
                if let Some(&(side, p)) = self.orders.get(&order.order_id) {
                    if side != OrderSide::Ask || p != price {
                        report
                            .violations
                            .push(IntegrityViolation::InvalidOrderIndex {
                                order_id: order.order_id,
                            });
                    }
                } else {
                    report
                        .violations
                        .push(IntegrityViolation::InvalidOrderIndex {
                            order_id: order.order_id,
                        });
                }
            }
        }

        // Check all index entries exist in queues
        for (&order_id, _) in self.orders.iter() {
            if !seen_orders.contains(&order_id) {
                report
                    .violations
                    .push(IntegrityViolation::InvalidOrderIndex { order_id });
            }
        }

        report
    }

    pub fn snapshot(&self) -> crate::snapshot::BookSnapshot {
        let best_bid = self.bids.iter().next_back().map(|(p, _)| *p);
        let best_ask = self.asks.iter().next().map(|(p, _)| *p);
        let spread_ticks = match (best_bid, best_ask) {
            (Some(b), Some(a)) => a.0 - b.0,
            _ => 0,
        };

        let total_bid_liquidity = self
            .bids
            .values()
            .flat_map(|l| l.orders.iter())
            .map(|o| o.remaining_quantity.0)
            .sum();
        let total_ask_liquidity = self
            .asks
            .values()
            .flat_map(|l| l.orders.iter())
            .map(|o| o.remaining_quantity.0)
            .sum();

        crate::snapshot::BookSnapshot {
            symbol: self.symbol.clone(),
            best_bid,
            best_ask,
            spread_ticks,
            total_bid_levels: self.bids.len() as u64,
            total_ask_levels: self.asks.len() as u64,
            total_bid_liquidity: astra_core::types::Quantity(total_bid_liquidity),
            total_ask_liquidity: astra_core::types::Quantity(total_ask_liquidity),
        }
    }

    pub fn submit(&mut self, mut incoming: Order) -> Vec<OrderEvent> {
        let mut events = Vec::new();

        if incoming.symbol != self.symbol {
            events.push(OrderEvent::Rejected {
                order_id: incoming.order_id,
                reason: "Symbol mismatch".to_string(),
            });
            return events;
        }

        if self.orders.contains_key(&incoming.order_id) {
            events.push(OrderEvent::Rejected {
                order_id: incoming.order_id,
                reason: "Duplicate order_id".to_string(),
            });
            return events;
        }

        if incoming.quantity.0 == 0 || incoming.remaining_quantity.0 == 0 {
            events.push(OrderEvent::Rejected {
                order_id: incoming.order_id,
                reason: "Zero quantity".to_string(),
            });
            return events;
        }

        events.push(OrderEvent::Accepted(incoming.clone()));

        // Matching Phase
        let mut fully_filled_prices = Vec::new();

        match incoming.side {
            OrderSide::Bid => {
                for (&ask_price, level) in self.asks.iter_mut() {
                    if incoming.remaining_quantity.0 == 0 {
                        break;
                    }
                    if incoming.order_type == OrderType::Limit && incoming.price < ask_price {
                        break; // Spread not crossed
                    }

                    // Match against this level
                    while let Some(mut resting) = level.orders.pop_front() {
                        if incoming.remaining_quantity.0 == 0 {
                            level.orders.push_front(resting);
                            break;
                        }

                        let match_qty = incoming
                            .remaining_quantity
                            .0
                            .min(resting.remaining_quantity.0);

                        incoming.remaining_quantity.0 -= match_qty;
                        resting.remaining_quantity.0 -= match_qty;

                        events.push(OrderEvent::TradeExecuted(TradeExecution {
                            resting_order_id: resting.order_id,
                            aggressive_order_id: incoming.order_id,
                            symbol: self.symbol.clone(),
                            match_price: ask_price,
                            matched_quantity: Quantity(match_qty),
                            liquidity_side: LiquiditySide::Maker,
                            timestamp_ns: incoming.timestamp_ns,
                        }));
                        events.push(OrderEvent::TradeExecuted(TradeExecution {
                            resting_order_id: resting.order_id,
                            aggressive_order_id: incoming.order_id,
                            symbol: self.symbol.clone(),
                            match_price: ask_price,
                            matched_quantity: Quantity(match_qty),
                            liquidity_side: LiquiditySide::Taker,
                            timestamp_ns: incoming.timestamp_ns,
                        }));

                        if resting.remaining_quantity.0 > 0 {
                            level.orders.push_front(resting);
                        } else {
                            self.orders.remove(&resting.order_id);
                        }
                    }

                    if level.orders.is_empty() {
                        fully_filled_prices.push(ask_price);
                    }
                }

                for p in fully_filled_prices {
                    self.asks.remove(&p);
                }
            }
            OrderSide::Ask => {
                for (&bid_price, level) in self.bids.iter_mut().rev() {
                    if incoming.remaining_quantity.0 == 0 {
                        break;
                    }
                    if incoming.order_type == OrderType::Limit && incoming.price > bid_price {
                        break; // Spread not crossed
                    }

                    while let Some(mut resting) = level.orders.pop_front() {
                        if incoming.remaining_quantity.0 == 0 {
                            level.orders.push_front(resting);
                            break;
                        }

                        let match_qty = incoming
                            .remaining_quantity
                            .0
                            .min(resting.remaining_quantity.0);

                        incoming.remaining_quantity.0 -= match_qty;
                        resting.remaining_quantity.0 -= match_qty;

                        events.push(OrderEvent::TradeExecuted(TradeExecution {
                            resting_order_id: resting.order_id,
                            aggressive_order_id: incoming.order_id,
                            symbol: self.symbol.clone(),
                            match_price: bid_price,
                            matched_quantity: Quantity(match_qty),
                            liquidity_side: LiquiditySide::Maker,
                            timestamp_ns: incoming.timestamp_ns,
                        }));
                        events.push(OrderEvent::TradeExecuted(TradeExecution {
                            resting_order_id: resting.order_id,
                            aggressive_order_id: incoming.order_id,
                            symbol: self.symbol.clone(),
                            match_price: bid_price,
                            matched_quantity: Quantity(match_qty),
                            liquidity_side: LiquiditySide::Taker,
                            timestamp_ns: incoming.timestamp_ns,
                        }));

                        if resting.remaining_quantity.0 > 0 {
                            level.orders.push_front(resting);
                        } else {
                            self.orders.remove(&resting.order_id);
                        }
                    }

                    if level.orders.is_empty() {
                        fully_filled_prices.push(bid_price);
                    }
                }

                for p in fully_filled_prices {
                    self.bids.remove(&p);
                }
            }
        }

        // Resting Phase (if not fully filled and Limit order)
        if incoming.remaining_quantity.0 > 0 {
            if incoming.order_type == OrderType::Limit {
                match incoming.side {
                    OrderSide::Bid => {
                        let level = self.bids.entry(incoming.price).or_default();
                        level.orders.push_back(incoming.clone());
                    }
                    OrderSide::Ask => {
                        let level = self.asks.entry(incoming.price).or_default();
                        level.orders.push_back(incoming.clone());
                    }
                }
                self.orders
                    .insert(incoming.order_id, (incoming.side, incoming.price));
            } else {
                events.push(OrderEvent::Cancelled {
                    order_id: incoming.order_id,
                    symbol: self.symbol.clone(),
                    reason: "Market order not fully filled".to_string(),
                });
            }
        }

        events
    }

    pub fn cancel(&mut self, order_id: u64) -> Vec<OrderEvent> {
        let mut events = Vec::new();
        let (side, price) = match self.orders.remove(&order_id) {
            Some(v) => v,
            None => {
                events.push(OrderEvent::Rejected {
                    order_id,
                    reason: "Order not found".to_string(),
                });
                return events;
            }
        };

        let level_opt = match side {
            OrderSide::Bid => self.bids.get_mut(&price),
            OrderSide::Ask => self.asks.get_mut(&price),
        };

        if let Some(level) = level_opt {
            let initial_len = level.orders.len();
            level.orders.retain(|o| o.order_id != order_id);
            if level.orders.len() < initial_len {
                events.push(OrderEvent::Cancelled {
                    order_id,
                    symbol: self.symbol.clone(),
                    reason: "User requested cancel".to_string(),
                });
            }

            if level.orders.is_empty() {
                match side {
                    OrderSide::Bid => self.bids.remove(&price),
                    OrderSide::Ask => self.asks.remove(&price),
                };
            }
        }

        events
    }

    pub fn modify(&mut self, order_id: u64, new_quantity: Quantity) -> Vec<OrderEvent> {
        let mut events = Vec::new();
        let (side, price) = match self.orders.get(&order_id) {
            Some(v) => *v,
            None => {
                events.push(OrderEvent::Rejected {
                    order_id,
                    reason: "Order not found".to_string(),
                });
                return events;
            }
        };

        let level_opt = match side {
            OrderSide::Bid => self.bids.get_mut(&price),
            OrderSide::Ask => self.asks.get_mut(&price),
        };

        if let Some(level) = level_opt {
            if let Some(order) = level.orders.iter_mut().find(|o| o.order_id == order_id) {
                if new_quantity.0 > order.remaining_quantity.0 {
                    events.push(OrderEvent::Rejected {
                        order_id,
                        reason: "Modify cannot increase quantity".to_string(),
                    });
                } else if new_quantity.0 == 0 {
                    // Equivalent to cancel
                    return self.cancel(order_id);
                } else {
                    order.remaining_quantity = new_quantity;
                    events.push(OrderEvent::Modified {
                        order_id,
                        symbol: self.symbol.clone(),
                        new_quantity,
                    });
                }
            }
        }

        events
    }
}
