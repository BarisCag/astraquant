use astra_core::events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata};
use astra_core::hashing::{hash_to_hex, DeterministicState};
use astra_core::replay::EventReducer;
use astra_core::serialization::serialize_canonical;
use astra_core::types::{Price, Quantity};
use astra_lob::engine::MatchingEngine;
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
        trader_id: 1,
        queue_position: Default::default(),
    }
}

fn main() {
    println!("AstraQuant astra-lob: Deterministic Replay Trace");
    println!("------------------------------------------------");

    // 1. Create a simulated journal of events
    let order1 = make_order(1, OrderSide::Bid, OrderType::Limit, 100, 10);
    let order2 = make_order(2, OrderSide::Ask, OrderType::Limit, 100, 5);

    let event1 = AstraEvent::new(
        1000,
        1,
        EventType::OrderSubmitted,
        serialize_canonical(&order1).unwrap(),
        PayloadMetadata::new(PayloadEncoding::Bincode, 1),
    );
    let event2 = AstraEvent::new(
        1001,
        2,
        EventType::OrderSubmitted,
        serialize_canonical(&order2).unwrap(),
        PayloadMetadata::new(PayloadEncoding::Bincode, 1),
    );

    let journal = vec![event1, event2];

    // 2. Pass 1
    let mut engine1 = MatchingEngine::new();
    for event in &journal {
        engine1.apply(event).unwrap();
    }
    let hash1 = engine1.state_hash();
    println!("Replay Pass 1 State Hash: {}", hash_to_hex(&hash1));
    println!("Engine Sequence ID:       {}", engine1.engine_sequence_id);

    // 3. Pass 2
    let mut engine2 = MatchingEngine::new();
    for event in &journal {
        engine2.apply(event).unwrap();
    }
    let hash2 = engine2.state_hash();
    println!("\nReplay Pass 2 State Hash: {}", hash_to_hex(&hash2));
    println!("Engine Sequence ID:       {}", engine2.engine_sequence_id);

    // 4. Verify Identity
    println!("\nDeterministic Equivalence: {}", hash1 == hash2);
}
