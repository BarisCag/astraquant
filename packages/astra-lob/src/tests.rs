#[cfg(test)]
mod tests {
    use crate::book::LimitOrderBook;
    use crate::engine::MatchingEngine;
    use crate::types::{Order, OrderEvent, OrderSide, OrderType};
    use astra_core::events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata};
    use astra_core::hashing::DeterministicState;
    use astra_core::serialization::serialize_canonical;
    use astra_core::types::{Price, Quantity};
    use std::time::Instant;

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

    #[test]
    fn test_fifo_priority() {
        let mut book = LimitOrderBook::new("BTCUSDT".to_string());

        // Add two bids at the same price
        book.submit(make_order(1, OrderSide::Bid, OrderType::Limit, 100, 10));
        book.submit(make_order(2, OrderSide::Bid, OrderType::Limit, 100, 10));

        // Submit a sell that crosses the spread, matching 15 quantity
        let events = book.submit(make_order(3, OrderSide::Ask, OrderType::Limit, 100, 15));

        // Expect 4 trade executions (2 pairs of Maker/Taker)
        let trades: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let OrderEvent::TradeExecuted(t) = e {
                    Some(t)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(trades.len(), 4);

        // Order 1 (first in) should be fully filled (10)
        assert_eq!(trades[0].resting_order_id, 1);
        assert_eq!(trades[0].matched_quantity.0, 10);
        assert_eq!(trades[0].liquidity_side, crate::types::LiquiditySide::Maker);

        // Order 2 (second in) should be partially filled (5)
        assert_eq!(trades[2].resting_order_id, 2);
        assert_eq!(trades[2].matched_quantity.0, 5);
        assert_eq!(trades[2].liquidity_side, crate::types::LiquiditySide::Maker);

        // Check book state
        let bid_level = book.bids.get(&Price(100)).unwrap();
        assert_eq!(bid_level.orders.len(), 1);
        assert_eq!(bid_level.orders[0].order_id, 2);
        assert_eq!(bid_level.orders[0].remaining_quantity.0, 5);
    }

    #[test]
    fn test_market_order() {
        let mut book = LimitOrderBook::new("BTCUSDT".to_string());
        book.submit(make_order(1, OrderSide::Ask, OrderType::Limit, 100, 10));
        book.submit(make_order(2, OrderSide::Ask, OrderType::Limit, 101, 10));

        // Buy 15 at market
        let events = book.submit(make_order(3, OrderSide::Bid, OrderType::Market, 0, 15));

        let trades: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let OrderEvent::TradeExecuted(t) = e {
                    Some(t)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(trades.len(), 4);
        assert_eq!(trades[0].match_price.0, 100);
        assert_eq!(trades[0].matched_quantity.0, 10);
        assert_eq!(trades[2].match_price.0, 101);
        assert_eq!(trades[2].matched_quantity.0, 5);

        // Market order should NOT be left in the book
        assert!(!book.orders.contains_key(&3));
    }

    #[test]
    fn test_cancel_order() {
        let mut book = LimitOrderBook::new("BTCUSDT".to_string());
        book.submit(make_order(1, OrderSide::Bid, OrderType::Limit, 100, 10));

        assert!(book.orders.contains_key(&1));
        assert!(book.bids.contains_key(&Price(100)));

        let events = book.cancel(1);

        let is_cancelled = events
            .iter()
            .any(|e| matches!(e, OrderEvent::Cancelled { order_id: 1, .. }));
        assert!(is_cancelled);

        assert!(!book.orders.contains_key(&1));
        assert!(!book.bids.contains_key(&Price(100)));

        // 3. Zero quantity
        use crate::invariants::IntegrityViolation;
        book.submit(make_order(1, OrderSide::Bid, OrderType::Limit, 100, 10));
        book.bids.get_mut(&Price(100)).unwrap().orders[0]
            .remaining_quantity
            .0 = 0;
        let report2 = book.validate_invariants();
        assert!(report2
            .violations
            .iter()
            .any(|v| matches!(v, IntegrityViolation::ZeroQuantityOrder { .. })));
    }

    #[test]
    fn test_modify_order() {
        let mut book = LimitOrderBook::new("BTCUSDT".to_string());
        book.submit(make_order(1, OrderSide::Bid, OrderType::Limit, 100, 10));

        let events = book.modify(1, Quantity(5));

        let is_modified = events.iter().any(|e| {
            matches!(
                e,
                OrderEvent::Modified {
                    order_id: 1,
                    new_quantity: Quantity(5),
                    ..
                }
            )
        });
        assert!(is_modified);

        let bid_level = book.bids.get(&Price(100)).unwrap();
        assert_eq!(bid_level.orders[0].remaining_quantity.0, 5);
    }

    #[test]
    fn test_determinism_identical_hash() {
        use crate::engine::MatchingEngine;
        use astra_core::replay::EventReducer;

        let mut engine1 = MatchingEngine::new();
        let mut engine2 = MatchingEngine::new();

        let order = make_order(1, OrderSide::Bid, OrderType::Limit, 100, 10);
        let payload = serialize_canonical(&order).unwrap();

        let event = AstraEvent::new(
            1000,
            1,
            EventType::OrderSubmitted,
            payload,
            PayloadMetadata::new(PayloadEncoding::Bincode, 1),
        );

        engine1.apply(&event).unwrap();
        engine2.apply(&event).unwrap();

        assert_eq!(engine1.state_hash(), engine2.state_hash());
    }

    #[test]
    fn test_benchmark_throughput() {
        let mut engine = MatchingEngine::new();

        let start = Instant::now();
        let num_orders = 10_000;

        for i in 0..num_orders {
            let side = if i % 2 == 0 {
                OrderSide::Bid
            } else {
                OrderSide::Ask
            };
            let order = make_order(i, side, OrderType::Limit, (100 + (i % 10)) as i64, 10);
            engine.get_or_create_book("BTCUSDT").submit(order);
        }

        let elapsed = start.elapsed();
        println!("Inserted {} orders in {:?}", num_orders, elapsed);

        let eps = (num_orders as f64) / elapsed.as_secs_f64();
        println!("Throughput: {:.2} events/sec", eps);
        assert!(eps > 10_00.0); // very conservative baseline
    }

    #[test]
    fn test_benchmark_snapshot_throughput() {
        let mut book = LimitOrderBook::new("BTCUSDT".to_string());
        for i in 0..1_000 {
            book.submit(make_order(
                i,
                OrderSide::Bid,
                OrderType::Limit,
                (100 + i % 10) as i64,
                10,
            ));
        }

        let start = Instant::now();
        let iters = 10_000;
        for _ in 0..iters {
            std::hint::black_box(book.snapshot());
        }
        let elapsed = start.elapsed().as_secs_f64();
        println!(
            "Snapshot Throughput: {:.2} snapshots/sec",
            iters as f64 / elapsed
        );
        assert!(iters as f64 / elapsed > 100.0);
    }

    #[test]
    fn test_benchmark_export_throughput() {
        use crate::export;
        let mut book = LimitOrderBook::new("BTCUSDT".to_string());
        for i in 0..1_000 {
            book.submit(make_order(
                i,
                OrderSide::Bid,
                OrderType::Limit,
                (100 + i % 10) as i64,
                10,
            ));
        }
        let snap = book.snapshot();

        let start = Instant::now();
        let iters = 10_000;
        let mut sink = std::io::sink();
        for _ in 0..iters {
            export::export_book_snapshot_csv(&mut sink, &snap).unwrap();
        }
        let elapsed = start.elapsed().as_secs_f64();
        println!("Export Throughput: {:.2} rows/sec", iters as f64 / elapsed);
        assert!(iters as f64 / elapsed > 100.0);
    }
}
