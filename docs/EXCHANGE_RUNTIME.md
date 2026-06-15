# AstraQuant Exchange Runtime

AstraQuant provides two distinct exchange runtimes to serve different research needs:

## Current Orchestration Runtime: `astra-exchange`
The `astra-exchange` crate provides the full-system authoritative orchestration layer for the AstraQuant suite.
This runtime seamlessly pipes execution events across all advanced deterministic subsystems without circular dependencies:
1. `astra-stream`: Normalizes incoming events.
2. `astra-risk`: Validates multi-tenant margin constraints deterministically.
3. `astra-lob`: Processes accepted limit orders through a rigorous FIFO matching engine.
4. `astra-portfolio`: Absorbs resulting `TradeExecuted` events to update average cost bases and portfolio snapshots.

This orchestrated pipeline is positioned purely as **"deterministic quantitative systems research infrastructure"**. It strictly forbids floating-point non-determinism, wall-clock time dependence, async network contamination, or fake AI execution claims.

## Legacy Runtime: `astra-core/src/exchange.rs`
The original `ExchangeRuntime` located in `astra-core` is preserved purely as a minimal deterministic runtime. It operates without `astra-lob` capabilities and serves as an important historical reference point for architectural evolution.
