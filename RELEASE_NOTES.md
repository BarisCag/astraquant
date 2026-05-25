# Release notes

## v0.1.0

### Summary

Public research prototype: deterministic event journal, replay engine, and wired exchange reducers.

### Implemented

- EventJournal, ReplayEngine, SnapshotManager
- ExchangeRuntime with matching, portfolio, ledger, risk
- StrategyRuntime with mean-reversion state
- Ops binary with Prometheus metrics HTTP
- CI: fmt, clippy, test, release build

### Limitations

- No WASM VM, no live feeds in default binary, no network consensus
- Python bindings not wired in astra-core