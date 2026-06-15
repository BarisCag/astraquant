use astra_lob::book::LimitOrderBook;
use astra_lob::types::OrderEvent;
use astra_core::types::Price;

pub struct LiquidityCollapseModel;

impl LiquidityCollapseModel {
    pub fn apply(book: &mut LimitOrderBook, max_orders_to_cancel: usize) -> Vec<OrderEvent> {
        let mut events = Vec::new();
        let mut to_cancel = Vec::new();
        
        // Cancel deepest liquidity first (lowest priority)
        // Bids: deep = lowest price
        for (_price, level) in book.bids.iter() {
            // level.orders are in FIFO order, so we remove the last ones first
            for order in level.orders.iter().rev() {
                if to_cancel.len() < max_orders_to_cancel {
                    to_cancel.push(order.order_id);
                } else {
                    break;
                }
            }
            if to_cancel.len() >= max_orders_to_cancel {
                break;
            }
        }

        // Asks: deep = highest price
        for (_price, level) in book.asks.iter().rev() {
            for order in level.orders.iter().rev() {
                if to_cancel.len() < max_orders_to_cancel {
                    to_cancel.push(order.order_id);
                } else {
                    break;
                }
            }
            if to_cancel.len() >= max_orders_to_cancel {
                break;
            }
        }

        for id in to_cancel {
            events.extend(book.cancel(id));
        }

        events
    }
}

pub struct SpreadExpansionModel;

impl SpreadExpansionModel {
    pub fn apply(book: &mut LimitOrderBook, tick_expansion: i64) -> Vec<OrderEvent> {
        let mut events = Vec::new();
        let mut order_modifications = Vec::new();

        // Widen bids: move them down by tick_expansion
        for (price, level) in book.bids.iter() {
            let new_price = Price::new(price.0 - tick_expansion);
            for order in level.orders.iter() {
                order_modifications.push((order.order_id, new_price, order.remaining_quantity, order.side));
            }
        }

        // Widen asks: move them up by tick_expansion
        for (price, level) in book.asks.iter() {
            let new_price = Price::new(price.0 + tick_expansion);
            for order in level.orders.iter() {
                order_modifications.push((order.order_id, new_price, order.remaining_quantity, order.side));
            }
        }

        // Apply modifications: Cancel and recreate
        // Since LimitOrderBook in astra-lob does not support native cancel-replace that changes price,
        // we must cancel and submit a new order. But we only have `book.submit` which requires a full Order struct.
        // Let's gather the full orders to resubmit first.
        let mut to_resubmit = Vec::new();
        
        for (id, new_price, _qty, _side) in order_modifications {
            // Find the original order to clone
            let orig = book.get_order(id).cloned();
            if let Some(mut o) = orig {
                events.extend(book.cancel(id));
                o.price = new_price;
                to_resubmit.push(o);
            }
        }

        for o in to_resubmit {
            events.extend(book.submit(o));
        }

        events
    }
}
