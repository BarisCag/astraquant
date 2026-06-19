# AstraQuant Trade — Technical Reference

## Architecture

AstraQuant Trade is built on the same deterministic
kernel as AstraQuant Research. Every market event
flows through the ExecutionGateway, gets assigned
a Sequence ID, and is written to the append-only
EventJournal before touching any business logic.

## Data Flow
Binance WebSocket

↓

ExecutionGateway (Sequence ID assigned)

↓

EventJournal (Blake3 footer)

↓

AstraKernel (deterministic event loop)

↓

┌─────────────┬──────────────┬─────────────┐

│ astra-trade │ astra-treasury│  astra-risk │

│ Paper Engine│ Cash/FX Ops  │ VaR/ES/Greeks│

└─────────────┴──────────────┴─────────────┘

↓

astra-alm (CVaR optimization, immunization)

↓

astra-api (REST + WebSocket, RBAC, audit)

## Determinism Guarantee

All Trade modules inherit the Research platform's
determinism guarantee:

Same market data sequence + same configuration
= identical state hash at every step

This applies to:
- Treasury cash flow forecasts
- VaR/ES calculations (deterministic Monte Carlo)
- ALM mismatch reports
- Hedge recommendations (DRL stub)

## RBAC Permission Matrix

| Endpoint | Trader | Risk | Treasurer | Auditor | Admin |
|----------|--------|------|-----------|---------|-------|
| GET /market/* | ✓ | ✓ | ✓ | ✓ | ✓ |
| GET /portfolio/* | ✓ | ✓ | ✓ | ✓ | ✓ |
| GET /treasury/* | ✗ | ✓ | ✓ | ✓ | ✓ |
| GET /risk/* | ✓ | ✓ | ✓ | ✓ | ✓ |
| GET /alm/mismatch | ✗ | ✓ | ✓ | ✓ | ✓ |
| POST /alm/hedge/approve | ✗ | ✗ | ✓ | ✗ | ✓ |
| POST /admin/killswitch | ✗ | ✓ | ✗ | ✗ | ✓ |
| GET /audit/* | ✗ | ✗ | ✗ | ✓ | ✓ |

## Compliance Readiness

Audit trail structure is designed for
ISAE 3402 / SOC 2 Type II review:
- Every API call logged with Blake3 chain
- Tamper-evident (hash break detected if modified)
- Append-only journal, never truncated
