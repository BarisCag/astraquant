# AstraQuant Replay Observability Philosophy

AstraQuant is designed as a deterministic execution infrastructure research platform. It is emphatically *not* a production HFT exchange, and it does not make artificial claims about microsecond networking or distributed matching. Instead, it guarantees 100% deterministic, mathematically auditable replay of market events.

## 1. Deterministic Diagnostics
Every interaction inside the `astra-lob` module (fills, cancels, modifications, invariants) updates the `ReplayDiagnostics` struct using pure integer arithmetic. This ensures that a simulation run on a developer laptop yields the exact same counters as a run on a CI server.

## 2. Replay-Safe Analytics
Execution trace analytics (such as maker/taker ratios or order depth peaks) are derived purely from canonical event journals. Because the matching engine avoids `async` or thread-pool non-determinism, the sequence clock (`engine_sequence_id`) serves as the true "time" inside the matching sandbox.

## 3. Why Float-Free Architecture Matters
Standard architectures often use `f64` for prices or aggregated metrics, introducing rounding errors that differ based on compilation flags or architecture. AstraQuant enforces integer representations (`Price`, `Quantity`, fixed-point counters), physically eliminating underflow/overflow non-determinism.

## 4. Why Stable Ordering Matters
The limit order book uses `BTreeMap` and `VecDeque`. We strictly avoid `HashMap`. This ensures that snapshots and book iterations always traverse prices and order queues in identical order, ensuring that CSV exports and structural hashes (`blake3`) remain perfectly reproducible.

## 5. CSV Export as Introspection
The `export.rs` module outputs snapshots, trade traces, and diagnostic telemetry purely through deterministic UTF-8 writes. No runtime timestamps (e.g., `chrono::Utc::now()`) are injected during export, ensuring the output is perfectly reproducible for data science tasks.
