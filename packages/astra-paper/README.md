# AstraQuant Trade: Paper Trading Engine (Phase 2)

This package implements the Paper Trading Engine for AstraQuant Trade. It connects to the `astra-venue` live market data feed and simulates execution, tracking P&L, positions, and enforcing circuit breaker guardrails. All outcomes are written directly to the `astra-core` deterministic journal, allowing for 100% accurate replay and auditability.

## How to Wire a New Strategy

1. Open `packages/astra-paper/src/strategy.rs`.
2. Create a new struct and implement the `Strategy` trait:

```rust
use crate::types::{MarketSnapshot, PaperOrder, PaperFill};
use crate::strategy::Strategy;

pub struct MyStrategy {
    // ... state variables ...
}

impl Strategy for MyStrategy {
    fn on_market_data(&mut self, snapshot: &MarketSnapshot) -> Option<Vec<PaperOrder>> {
        // Execute logic on every market tick.
        // Return Some(orders) to submit new orders.
        None
    }

    fn on_fill(&mut self, fill: &PaperFill) {
        // Called whenever the engine executes your order.
    }

    fn on_clock(&mut self, interval_ns: u64) -> Option<Vec<PaperOrder>> {
        // Called to allow time-based logic independently of market data.
        None
    }
}
```

3. Update `packages/astra-trade/src/main.rs` to instantiate your strategy and pass it into the `PaperEngine`.

## How to Run the Deterministic Replay Test

To prove identical state parity between live paper runs and historical replay, run the determinism test suite.

```bash
cargo test -p astra-paper --test deterministic_replay_test
```

This test simulates processing the `flash_crash_2010.astra_ds` dataset twice, comparing the final Blake3 state hashes of the journals to ensure branchless execution and absolute determinism.

## How to Read P&L from the Journal

Every action in the paper trading engine is captured as an `AstraEvent` and appended to the `.astra_jl` journal. 
Because `astra-paper` uses the existing `ExecutionGateway`, you can replay the journal using `astra-core` or `astra-audit` tools.

1. **Read `OrderFilled` events:** These contain the `PaperFill` payloads.
2. **Calculate Realized P&L:** Calculate the difference between consecutive fill prices for opposite sides using FIFO matching.
3. **Calculate Unrealized P&L (Mark-to-Market):** Cross-reference open quantities against the latest `MarketTick` price recorded in the journal.
