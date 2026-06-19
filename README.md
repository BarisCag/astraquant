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

## AstraQuant Trade

A live institutional trading and treasury OS 
built on top of AstraQuant's deterministic core.

### Modules

| Module | Phase | Description |
|--------|-------|-------------|
| astra-venue | 1 | Live Binance WebSocket feed |
| astra-trade | 2 | Paper trading engine |
| astra-treasury | 3 | Multi-currency treasury operations |
| astra-risk | 4 | VaR, Expected Shortfall, Greeks |
| astra-alm | 5 | Asset-Liability Management, CVaR optimization |
| astra-api | 6 | Institutional REST + WebSocket API |

### Quick Start

```bash
# Start the institutional API (live mode)
cargo run -p astra-api

# Start in demo mode (sanitized data)
DEMO_MODE=true cargo run -p astra-api

# Health check
curl http://localhost:8080/health

# Run live Binance feed
cargo run -p astra-trade
```

### API Endpoints
GET  /health                  → System status + mode

GET  /market/snapshot         → Live market data

GET  /portfolio               → Current positions + P&L

GET  /treasury/cashflow       → 30-day cash flow forecast

GET  /treasury/exposure       → FX exposure by tenor

GET  /risk/var                → VaR at 95% and 99%

GET  /risk/es                 → Expected Shortfall 97.5%

GET  /risk/greeks             → Delta, Gamma, Vega, Theta, Rho

GET  /alm/mismatch            → Duration gap by tenor/currency

POST /alm/hedge/approve       → Approve hedge recommendation

POST /admin/killswitch        → Emergency halt

WS   /ws/stream               → Real-time event stream

### Security

- JWT authentication (24h expiry)
- 5-role RBAC (Trader, RiskManager, Treasurer, Auditor, Admin)
- Blake3-chained audit trail (tamper-evident)
- Rate limiting: 100 req/min, burst 20
- Demo mode: sanitized data for showcase
