# AstraQuant `astra-risk` Phase 1: Deterministic Risk Infrastructure

AstraQuant's `astra-risk` module is explicitly designed as **deterministic event-driven risk infrastructure research**. It evaluates orders deterministically outside the execution flow and prevents violations from entering the core `astra-lob` matching cycle.

## Deterministic Risk Philosophy

In high-performance quantitative infrastructure, risk bounds must be as replay-safe as the matching engine itself.

### Why Timestamps Are Forbidden
Network jitter, clock drift, and OS scheduler interrupts mean that relying on wall-clock timestamps (`std::time::Instant` or `chrono`) for velocity limits creates a system that can *pass* in production but *fail* in replay, or vice-versa. 
`astra-risk` relies purely on `engine_sequence_id` to enforce deterministic velocity windows. A sequence window eviction logic ensures that a given journal state will *always* evict orders exactly at the same sequence index across any machine or architecture.

### Replay-Safe State Evolution
Any order rejection triggers a `RiskViolation` event. Rejected orders are never forwarded to the matching engine. The deterministic state hash of `RiskEngine` includes:
- Multi-tenant `TraderExposure` tracking
- Replay-safe `VelocityWindow` queues
- Deterministic BTreeMap routing and serialization
Because these state structures are exclusively integer-bound and serializable via `bincode`, the entire risk profile evolves uniformly.

### Limitations
This is a research prototype targeting determinism, not a drop-in replacement for an institutional production RMS or real-time clearing system. Current limitations include:
- **No Mark-to-Market (MTM) Pricing**: PnL is based on strictly realized fills, not continuous BBO marking.
- **No Value-at-Risk (VaR) or Greeks**: Non-linear models and covariance matrices require floating-point estimations which violate BLAKE3 deterministic hashing.
- **No Margin Liquidations**: Phase 1 enforces pre-trade limits but lacks a secondary background liquidation engine.
