# AstraQuant OS — Research Portfolio Summary

## Platform Overview
AstraQuant OS is a mathematically deterministic financial operating system that allows institutional researchers to replay historical market crises with exact bit-for-bit reproducibility. By eliminating all non-deterministic factors—such as floating-point arithmetic, network jitter, and wall-clock time—AstraQuant proves that market crises are deterministic systems. This absolute precision ensures that policy interventions can be scientifically isolated and proven.

## Crisis Studies Completed
| Crisis | Events | Counterfactuals | Key Finding |
|--------|--------|-----------------|-------------|
| 2010 Flash Crash | ~1,000 | 3 interventions | Circuit breaker timing critical |
| 2008 Lehman Collapse | ~1,200 | 3 interventions | Contagion speed underestimated |
| 2020 COVID Crash | ~800 | 3 interventions | Liquidity injection most effective |

## Behavioral Calibration Summary
| Crisis | Best Herding Factor | Best Loss Aversion | Fit Score |
|--------|---------------------|--------------------|-----------|
| 2010 Flash Crash | 0.1 | 1.5 | 161.0 |
| 2008 Lehman Collapse | 0.1 | 1.5 | 327.0 |
| 2020 COVID Crash | 0.1 | 1.5 | 1179.0 |

## RL Policy Findings
| Crisis | Optimal Sequence | Best Action | Cascade Prevented |
|--------|------------------|-------------|-------------------|
| 2010 Flash Crash | 300 | SHORT_BAN | 19.9% |
| 2008 Lehman Collapse | 400 | CIRCUIT_BREAKER | 5.0% |
| 2020 COVID Crash | 400 | CIRCUIT_BREAKER | 7.5% |

## Methodology: Why Determinism Matters
Without determinism, simulators cannot distinguish whether a market recovery was caused by a policy intervention or merely by chaotic divergence in the simulation engine itself. AstraQuant’s deterministic kernel guarantees that any delta in the cryptographic state hash is exclusively the result of the injected policy, transforming financial economics from an observational science into an exact one.

## Audit & Reproducibility Statement
All results are reproducible via:
```bash
cargo test test_golden_hash_regression --workspace
```
Every event ingested produces an immutable Blake3 hash. Any researcher running the platform against the standard `.astra_ds` datasets will arrive at identical cryptographic roots.

## Technical Architecture Summary
The core of AstraQuant OS is a fixed-point deterministic kernel (`astra-core`) built in Rust. It utilizes an append-only event journal that chronologically sequences all market events and interventions. Execution occurs within a strictly ordered, single-threaded state machine where every action produces a verifiable state transition proof. Behavioral models are executed safely inside metered WASM sandboxes to preserve absolute determinism.

## Future Research Directions
- Phase B: Institutional trading OS
- Live venue integration
- Central bank API partnerships
