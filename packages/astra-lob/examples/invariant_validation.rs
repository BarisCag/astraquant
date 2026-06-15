use astra_core::types::{Price, Quantity};
use astra_lob::book::LimitOrderBook;
use astra_lob::types::{Order, OrderSide, OrderType};

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
        trader_id: 1, queue_position: Default::default(),
    }
}

fn main() {
    println!("AstraQuant astra-lob: Invariant Validation");
    println!("------------------------------------------");

    let mut book = LimitOrderBook::new("BTCUSDT".to_string());

    // Submit valid orders
    book.submit(make_order(1, OrderSide::Bid, OrderType::Limit, 100, 10));
    book.submit(make_order(2, OrderSide::Ask, OrderType::Limit, 105, 10));

    let report1 = book.validate_invariants();
    println!("1. Clean book state: {:?}", report1.is_clean());

    // Intentionally corrupt book state to demonstrate detection
    println!("\n2. Corrupting book state (injecting zero quantity)");
    book.bids.get_mut(&Price(100)).unwrap().orders[0]
        .remaining_quantity
        .0 = 0;

    let report2 = book.validate_invariants();
    println!("Book is clean? {}", report2.is_clean());
    for violation in report2.violations {
        println!(" - Violation Detected: {:?}", violation);
    }
}
