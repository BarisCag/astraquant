# AstraQuant Phase 4B: Deterministic Portfolio Accounting Research Infrastructure

`astra-portfolio` is the deterministic portfolio and position accounting subsystem for AstraQuant. Built with mathematical strictness, it models multi-tenant tracking of open positions, realized PnL, and unrealized PnL via average cost basis accounting.

## Architectural Principles

### Zero-Float Safety
All quantities, average entry prices, and mark prices are tracked strictly as fixed-point integers (usually `i64`). Floating point (`f32`/`f64`) operations are explicitly forbidden in this module. This ensures that PnL calculations are exactly reproducible on any machine architecture.

### Replay-Safe Mark Pricing
Unrealized PnL is computed from deterministic mark prices derived *exclusively* from replay events (such as `TradeSettled`). There are no external price feed polling loops or wall-clock dependencies, ensuring perfect replay identity. A given journal file will always yield identically valued portfolio snapshots.

### Isolated Multi-Tenant Topology
The system avoids single-portfolio assumptions. Through a hierarchical `BTreeMap<trader_id, BTreeMap<symbol, Position>>` structure, an infinite amount of independent traders can be modeled and independently snapshotted. BTreeMap iterators ensure that portfolio snapshots traverse assets deterministically, preventing hash sequence drifting.

### Honest Representation
This module is positioned as **"deterministic portfolio accounting research infrastructure"**. It does not constitute prime brokerage software. Complex models such as FIFO tax accounting, options Greeks, funding rates, or active financing margin sweeps are outside the deterministic domain of Phase 4B.

## PnL Flips & Crossing Zero
The core Position Engine handles inventory flips gracefully. If a trader holds Long 10, and issues a Short 15 order:
1. 10 shares are closed against the average entry price, crystallizing the realized PnL for those shares.
2. The remaining 5 shares establish a new short position (net_quantity = -5) at the new execution price, wiping out the previous average cost basis and establishing a new one.

This exactness guarantees that exposure invariants remain consistent during aggressive algorithmic simulations.
