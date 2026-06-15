# AstraQuant Research Tooling

AstraQuant is a **deterministic quantitative systems research infrastructure**, not a production brokerage exchange or HFT trading stack. The design of `astra-inspect` ensures that all analytics are mathematically pure, strictly integer-based, and perfectly reproducible.

## Deterministic Analytics Philosophy

1. **Zero Floating Point Math**: Floats produce different rounding outcomes on different CPU architectures. By utilizing strictly integer arithmetic (such as PPM—parts per million—for ratios) and integer-based types (like `u64`), AstraQuant maintains absolute multi-platform stability.
2. **Stable Iteration Order**: We use `BTreeMap` instead of `HashMap` everywhere. All exports (JSON, CSV, Timeline) iterate over identical memory structures in the exact same order on every platform.
3. **Replay-Safe Observability**: The `ReplayInspector` never mutates `ExchangeRuntime`. It peeks into the underlying state, tracks the events deterministically, and preserves isolation.

## Institutional Research Workflow

`astra-inspect` provides the tools to reconstruct exact matching conditions:
- Track Risk Rejections cleanly mapped to sequence IDs.
- Watch order queue depths change in response to arrivals.
- Review Maker/Taker ratios and precise spread conditions per millisecond.
- Produce deterministic Sequence Diagrams and ASCII orderbooks to deeply analyze market microstructure phenomena.
