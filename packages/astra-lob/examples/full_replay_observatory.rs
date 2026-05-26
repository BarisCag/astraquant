use astra_core::events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata};
use astra_core::hashing::{hash_to_hex, DeterministicState};
use astra_core::replay::EventReducer;
use astra_core::serialization::serialize_canonical;
use astra_core::types::{Price, Quantity};
use astra_lob::engine::MatchingEngine;
use astra_lob::export;
use astra_lob::types::{Order, OrderSide, OrderType};
use std::io;

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
    }
}

fn main() {
    println!("AstraQuant astra-lob: Institutional Replay Observatory");
    println!("======================================================");

    let mut engine = MatchingEngine::new();

    let journal = vec![
        make_order(1, OrderSide::Bid, OrderType::Limit, 100, 10),
        make_order(2, OrderSide::Bid, OrderType::Limit, 99, 10),
        make_order(3, OrderSide::Ask, OrderType::Limit, 105, 5),
        make_order(4, OrderSide::Ask, OrderType::Limit, 100, 5), // Crosses bid 1
    ];

    let mut sequence_id = 1;
    for order in journal {
        let event = AstraEvent::new(
            1000,
            sequence_id,
            EventType::OrderSubmitted,
            serialize_canonical(&order).unwrap(),
            PayloadMetadata::new(PayloadEncoding::Bincode, 1),
        );
        engine.apply(&event).unwrap();

        let book = engine.get_or_create_book("BTCUSDT");
        let snap = book.snapshot();
        println!(
            "\n[Seq {}] Engine Clock: {}",
            sequence_id, engine.engine_sequence_id
        );
        println!(
            "Best Bid: {:?} | Best Ask: {:?} | Spread: {}",
            snap.best_bid.map(|p| p.0),
            snap.best_ask.map(|p| p.0),
            snap.spread_ticks
        );

        sequence_id += 1;
    }

    println!("\nExporting Diagnostics to STDOUT:");
    println!("--------------------------------");
    export::export_replay_diagnostics_csv(&mut io::stdout(), &engine.diagnostics).unwrap();

    let book = engine.get_or_create_book("BTCUSDT");
    let snap = book.snapshot();
    println!("\nExporting Final Snapshot to STDOUT:");
    println!("-----------------------------------");
    export::export_book_snapshot_csv(&mut io::stdout(), &snap).unwrap();

    let final_hash = engine.state_hash();
    println!("\n[FINAL BLAKE3 STATE HASH]: {}", hash_to_hex(&final_hash));
}
