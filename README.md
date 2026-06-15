# AstraQuant OS
### Deterministic Financial Infrastructure Research Platform

> "Financial crises aren't random — they're deterministic systems.
>  AstraQuant proves it."

## What This Is

AstraQuant OS is a production-grade, mathematically deterministic 
financial operating system that replays historical crises with 
bit-for-bit reproducibility and tests the effect of policy 
interventions.

This is not a trading bot.
This is not a backtesting framework.
This is financial systems infrastructure — built the way 
institutional systems should be built.

## What Makes It Different

Most financial simulators are non-deterministic.
Same code → different run → different result.
This makes rigorous policy research impossible.

AstraQuant enforces absolute determinism at the kernel level:
- No floating point arithmetic (fixed-point integers only)
- No wall-clock time (logical sequence IDs only)  
- No randomness (all seeds are deterministic inputs)
- Every state transition produces a Blake3 cryptographic hash
- Replay of identical inputs must produce identical hash output

This is a mathematical guarantee, not a claim.

## Research Outputs

### Crisis Studies
| Crisis | Events | Counterfactuals | Key Finding |
|--------|--------|-----------------|-------------|
| 2010 Flash Crash | ~1,000 | 3 interventions | Circuit breaker timing critical |
| 2008 Lehman Collapse | ~1,200 | 3 interventions | Contagion speed underestimated |
| 2020 COVID Crash | ~800 | 3 interventions | Liquidity injection most effective |

### Policy Counterfactual Matrix
9 scenario combinations (3 crises × 3 interventions):
- CircuitBreakerHalt
- LiquidityInjection  
- ShortSellingBan

Each produces: baseline hash, intervention hash, 
price delta, cascade events prevented.

### Behavioral Analysis
5 WASM agent models calibrated against historical data:
- Herding (cascade amplification)
- Prospect Theory / Loss Aversion (Kahneman λ)
- Anchoring (institutional price memory)
- Salience (retail panic response)
- Liquidity Withdrawal (market maker flight)

### RL Policy Optimization
Optimal intervention timing identified per crisis scenario
via deterministic Q-table (1000 episodes, same seed = same result).

## Architecture
Non-Deterministic World

↓

ExecutionGateway (assigns Sequence ID, timestamps)

↓

Append-Only EventJournal (Blake3 footer)

↓

AstraKernel (deterministic event loop)

├── MatchingEngine (Price-Time Priority LOB)

├── PositionEngine (P&L, settlement)

├── RiskEngine (pre-trade gates)

└── WASM Sandbox (behavioral agents, gas metered)

↓

StateTransitionProof (auto-generated per event)

↓

MerkleTree (root every 100 events)

↓

Blake3 Audit Trail (cryptographically certifiable)

## Verification

```bash
# Clone and run
git clone https://github.com/[you]/astraquant
cd astraquant
docker-compose up --build

# Verify determinism
cargo test test_three_run_parity --workspace
# Expected: 3 identical state hashes

# Run golden hash regression
cargo test test_golden_hash_regression --workspace
# Expected: all 3 crisis hashes match pinned values

# Replay Flash Crash
astra-research --dataset flash_crash_2010.astra_ds \
               --output results/flash_crash/
```

## Research API
GET  /api/datasets              → list crisis datasets

POST /api/replay                → run deterministic replay

POST /api/counterfactual        → inject policy intervention

POST /api/behavioral            → calibrate behavioral agents

GET  /api/certification/{name}  → Blake3 verified audit report

## Test Suite
- 46+ tests passing
- Determinism verified: 3× replay parity
- Golden hashes pinned in CI (PR fails if hash changes)
- Mid-write crash recovery validated
- Symbolic divergence detection active

## Technical Stack
Rust · Axum · WASM · Blake3 · Prometheus · Grafana · Docker

## License
MIT

---
*Built as independent research alongside Economics (BSc), 
Çukurova University. Author pursuing MMF, Goethe University 
Frankfurt.*