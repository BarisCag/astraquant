# Limit Order Book (astra-lob) Design

`astra-lob` is the deterministic limit order book and matching engine layer of AstraQuant OS. It sits inside the `astra-core` quarantine boundary and processes purely deterministic `AstraEvent` objects to compute trade executions.

## 1. Matching Algorithm

The matching engine employs strict **Price-Time (FIFO) Priority**:

*   **Price Priority**: Better prices are executed first.
    *   Bids (Buy) are ordered highest price to lowest.
    *   Asks (Sell) are ordered lowest price to highest.
*   **Time Priority**: At a given price level, orders are executed in the exact order they arrived (FIFO).

When a new order is submitted:
1.  **Spread Crossing Check**: The incoming order is evaluated against the opposing side of the book.
2.  **Partial / Full Fills**: If it crosses, it matches against resting orders sequentially. `TradeExecution` events are emitted. Fully filled resting orders are dropped, partially filled resting orders remain at the front of their price level queue.
3.  **Resting**: If the incoming order is a `Limit` order and has `remaining_quantity > 0`, it is appended to the back of the queue at its specified price level.
4.  **Market Orders**: If a `Market` order cannot be fully filled, the unfilled remainder is cancelled.

## 2. Memory Model & Complexity

To satisfy absolute determinism and efficient scaling, `astra-lob` avoids all non-deterministic collections (like `HashMap`).

*   **Price Levels**: `BTreeMap<Price, PriceLevel>`
    *   Finding the best price: O(log N) where N is the number of active price levels.
    *   Iteration: Strict deterministic sorted order.
*   **Order Queues**: `VecDeque<Order>` inside `PriceLevel`.
    *   FIFO execution: O(1) `pop_front()`.
    *   Resting insertion: O(1) `push_back()`.
*   **Order Index**: `BTreeMap<order_id, (OrderSide, Price)>`
    *   Cancels & Modifications require finding the order's price level without a full scan: O(log M) where M is total resting orders.

## 3. Determinism Guarantees

*   **No Floating Point**: All prices and quantities use explicit scaling with fixed-point `i64` or `u64` values. `Price` and `Quantity` wrappers prevent accidental mixing.
*   **No Asynchrony**: The `MatchingEngine::apply()` method is fully synchronous and blocking. It never waits on external I/O.
*   **Composite State Hashing**: `LimitOrderBook` and `MatchingEngine` implement `DeterministicState`. The hash is a BLAKE3 digest of the canonical `bincode` serialization of the `BTreeMap` structures. Because `BTreeMap` is inherently sorted, the serialization and resulting hash are mathematically identical across any CPU architecture or OS for the same sequence of events.

## 4. Known Limitations

*   **Memory Footprint**: Keeping all orders in memory could become a bottleneck for million-order datasets without implementing a periodic garbage collection or snapshot eviction strategy (though `astra-core` snapshotting solves this at the whole-engine level).
*   **Single-Threaded**: The matching engine for a single symbol is strictly single-threaded. This is an intentional constraint to preserve deterministic ordering. Parallelism must be achieved by sharding across different symbols (which `astra-stream` does during ingestion).
*   **No Stop/Iceberg Orders**: This iteration implements basic `Limit` and `Market` semantics. Advanced conditional order types (Stop-Loss, Iceberg/Hidden, Trailing) are omitted for simplicity but can be added natively to the `OrderType` enum.
