use crate::orderbook::{
    LimitOrderMatchedPayload, LimitOrderPlacedPayload, OrderBook, OrderNode, OrderSide,
};
use crate::types::Quantity;
use std::cmp::min;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MatchingEngine {
    pub book: OrderBook,
}

impl MatchingEngine {
    pub fn new(symbol: String) -> Self {
        Self {
            book: OrderBook::new(symbol),
        }
    }

    pub fn process_limit_order(
        &mut self,
        payload: &LimitOrderPlacedPayload,
    ) -> Vec<LimitOrderMatchedPayload> {
        let mut matches = Vec::new();
        let mut remaining_qty = payload.quantity;

        match payload.side {
            OrderSide::Bid => {
                let mut empty_levels = Vec::new();
                let mut filled_orders = Vec::new();

                for (&ask_price, level) in self.book.asks.iter_mut() {
                    if payload.price < ask_price {
                        break;
                    }

                    for node in level.iter_mut() {
                        if remaining_qty.0 == 0 {
                            break;
                        }

                        let match_qty = min(remaining_qty.0, node.quantity.0);
                        matches.push(LimitOrderMatchedPayload {
                            maker_order_id: node.order_id,
                            taker_order_id: payload.order_id,
                            match_price: ask_price,
                            matched_quantity: Quantity::new(match_qty),
                        });

                        remaining_qty = Quantity::new(remaining_qty.0 - match_qty);
                        node.quantity = Quantity::new(node.quantity.0 - match_qty);

                        if node.quantity.0 == 0 {
                            filled_orders.push(node.order_id);
                        }
                    }

                    level.retain(|n| n.quantity.0 > 0);
                    if level.is_empty() {
                        empty_levels.push(ask_price);
                    }

                    if remaining_qty.0 == 0 {
                        break;
                    }
                }

                for p in empty_levels {
                    self.book.asks.remove(&p);
                }
                for id in filled_orders {
                    self.book.order_index.remove(&id);
                }

                if remaining_qty.0 > 0 {
                    let node = OrderNode {
                        order_id: payload.order_id,
                        quantity: remaining_qty,
                    };
                    self.book.bids.entry(payload.price).or_default().push(node);
                    self.book
                        .order_index
                        .insert(payload.order_id, (OrderSide::Bid, payload.price));
                }
            }
            OrderSide::Ask => {
                let mut empty_levels = Vec::new();
                let mut filled_orders = Vec::new();

                for (&bid_price, level) in self.book.bids.iter_mut().rev() {
                    if payload.price > bid_price {
                        break;
                    }

                    for node in level.iter_mut() {
                        if remaining_qty.0 == 0 {
                            break;
                        }

                        let match_qty = min(remaining_qty.0, node.quantity.0);
                        matches.push(LimitOrderMatchedPayload {
                            maker_order_id: node.order_id,
                            taker_order_id: payload.order_id,
                            match_price: bid_price,
                            matched_quantity: Quantity::new(match_qty),
                        });

                        remaining_qty = Quantity::new(remaining_qty.0 - match_qty);
                        node.quantity = Quantity::new(node.quantity.0 - match_qty);

                        if node.quantity.0 == 0 {
                            filled_orders.push(node.order_id);
                        }
                    }

                    level.retain(|n| n.quantity.0 > 0);
                    if level.is_empty() {
                        empty_levels.push(bid_price);
                    }

                    if remaining_qty.0 == 0 {
                        break;
                    }
                }

                for p in empty_levels {
                    self.book.bids.remove(&p);
                }
                for id in filled_orders {
                    self.book.order_index.remove(&id);
                }

                if remaining_qty.0 > 0 {
                    let node = OrderNode {
                        order_id: payload.order_id,
                        quantity: remaining_qty,
                    };
                    self.book.asks.entry(payload.price).or_default().push(node);
                    self.book
                        .order_index
                        .insert(payload.order_id, (OrderSide::Ask, payload.price));
                }
            }
        }

        matches
    }

    pub fn cancel_order(&mut self, order_id: u64) -> bool {
        if let Some((side, price)) = self.book.order_index.remove(&order_id) {
            let level = match side {
                OrderSide::Bid => self.book.bids.get_mut(&price),
                OrderSide::Ask => self.book.asks.get_mut(&price),
            };

            if let Some(nodes) = level {
                nodes.retain(|n| n.order_id != order_id);
                if nodes.is_empty() {
                    match side {
                        OrderSide::Bid => self.book.bids.remove(&price),
                        OrderSide::Ask => self.book.asks.remove(&price),
                    };
                }
            }
            true
        } else {
            false
        }
    }
}
