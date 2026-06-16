#![allow(clippy::all)]
use astra_core::types::{Price, Quantity};
use astra_lob::book::LimitOrderBook;
use astra_lob::types::{LiquiditySide, Order, OrderEvent, OrderSide, OrderType};

fn make_order(
    order_id: u64,
    side: OrderSide,
    order_type: OrderType,
    price: i64,
    quantity: u64,
) -> Order {
    Order {
        order_id,
        symbol: "BTCUSDT".to_string(),
        side,
        order_type,
        price: Price(price),
        quantity: Quantity(quantity),
        remaining_quantity: Quantity(quantity),
        timestamp_ns: 1000,
        trader_id: 1,
        queue_position: Default::default(),
    }
}

fn main() {
    println!("AstraQuant astra-lob: Maker/Taker Execution Semantics");
    println!("----------------------------------------------------");

    let mut book = LimitOrderBook::new("BTCUSDT".to_string());

    // 1. Maker provides liquidity
    println!("1. Submitting Maker Order: Sell 10 BTC @ $100");
    let resting_order = make_order(1, OrderSide::Ask, OrderType::Limit, 100, 10);
    book.submit(resting_order);

    // 2. Taker consumes liquidity
    println!("2. Submitting Taker Order: Buy 5 BTC @ $100");
    let aggressive_order = make_order(2, OrderSide::Bid, OrderType::Limit, 100, 5);
    let events = book.submit(aggressive_order);

    println!("\nExecution Trace:");
    for event in events {
        match event {
            OrderEvent::TradeExecuted(trade) => {
                let role = match trade.liquidity_side {
                    LiquiditySide::Maker => "Maker (Resting)",
                    LiquiditySide::Taker => "Taker (Aggressive)",
                };
                println!(
                    " - Trade Match: {} filled {} @ {} (Role: {})",
                    if trade.liquidity_side == LiquiditySide::Maker {
                        trade.resting_order_id
                    } else {
                        trade.aggressive_order_id
                    },
                    trade.matched_quantity.0,
                    trade.match_price.0,
                    role
                );
            }
            _ => {}
        }
    }
}
