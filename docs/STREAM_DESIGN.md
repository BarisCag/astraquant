# AstraQuant Stream Design

The `astra-stream` subsystem is the I/O quarantine layer for market data in AstraQuant OS. It handles all non-deterministic external interactions (network, wall-clock time) and distills them into purely deterministic sequences of bits.

## 1. The Quarantine Boundary

`astra-core` is purely deterministic and contains no `tokio`, no networking, and no floating-point math. `astra-stream` acts as a firewall protecting this boundary.

*   **Async/Tokio**: Confined entirely to `astra-stream/src/ingestion.rs`.
*   **Networking**: Confined to `BinanceTradeStream` via `tokio-tungstenite`.
*   **Decimal Parsing**: Market data prices (e.g. `"29345.67000000"`) are extracted directly from JSON as strings, avoiding the `f64` representation completely. They are parsed directly into integer fixed-point values (`i64`) using `parse_decimal_fixed()`.

## 2. Multi-Symbol Ingestion Architecture

`astra-stream` handles multiple concurrent WebSocket streams using a bounded channel architecture.

1.  **Per-Symbol WebSockets**: Each symbol (e.g. `btcusdt`, `ethusdt`) gets a dedicated `tokio::spawn` task maintaining a WebSocket connection.
2.  **Bounded Channels**: Each task parses JSON trades into `SymbolEvent` structs and sends them over a dedicated `tokio::sync::mpsc` channel (capacity 1024).
3.  **Unified Journal Writer**: A single coordinator task drains all channels and serializes writes to disk.
4.  **Backpressure**: Because channels are bounded, if disk I/O stalls, the WebSocket tasks will block on `send()`, preventing unbounded memory growth. The memory footprint is strictly bounded.

## 3. Time-Bucketed Journal Rotation

To prevent monolithic, unmanageable journal files, `astra-stream` implements fully deterministic, time-bucketed journal rotation.

*   **Filename Format**: `{symbol}_{YYYY}_{MM}_{DD}_{HH}.astra_jl` (e.g. `btcusdt_2024_01_15_09.astra_jl`).
*   **Epoch-based**: Uses Unix epoch seconds to compute the UTC Gregorian hour bucket entirely within `astra-stream` (no `chrono` dependency).
*   **Synchronous Rollover**: On every event commit, the writer checks if the current hour bucket has changed. If so, it synchronously flushes the old journal and opens the new one. There is no background rotation thread.

## 4. Replay Directory Mode & Integrity

The `ReplayEngine` can replay entire directories of rotated journals.

*   **Deterministic Ordering**: Directories are scanned for `.astra_jl` files, which are then sorted lexicographically. Because of the `{symbol}_{YYYY}...` format, lexicographic order perfectly matches chronological order for a given symbol.
*   **Continuous Hash Chain**: The BLAKE3 state hash accumulates continuously across file boundaries, producing a single final deterministic hash for the entire directory replay.
*   **Integrity Enforcement**: Replay enforces strict continuity rules:
    *   **DuplicateTradeId**: Detects duplicate trades.
    *   **SequenceGap**: Detects missing `sequence_id`s in the `.astra_jl` framework.
    *   **OutOfOrderTimestamp**: Enforces monotonic exchange timestamps.
    *   **ReconnectGap**: Detects if a `trade_id` jumps by more than 1 (meaning the WebSocket missed trades during a disconnect).
    *   **StreamDiscontinuity**: Detects silent drops where the exchange timestamp gaps by more than 60 seconds.

## 5. Benchmarking & Metrics

The subsystem measures its own performance.

*   **Ingestion Metrics**: Exposes Prometheus counters/gauges for `events_ingested`, `events_per_second`, `websocket_reconnects`, `journal_bytes_written`, `bytes_per_second`, and `last_write_latency_us`.
*   **Replay Reporting**: Directory replays output structural reports showing total events, total time, throughput (`events/sec`), and a cryptographic hash. Replay summaries can be exported to CSV using `--bench-csv`.
