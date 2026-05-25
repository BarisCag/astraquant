# FLOATING_POINT_POLICY.md â€” AstraQuant OS Constitutional Doctrine

## Absolute Prohibition

```
The AstraQuant OS event layer FORBIDS floating-point financial state.

All prices, quantities, and monetary values MUST use deterministic
fixed-point encoding.
```

This is permanent constitutional doctrine, not a guideline.

---

## Approved Representations

| Domain | Type | Scale | Example |
|--------|------|-------|---------|
| Prices | `i64` | 4 decimal places | `1234500` = `123.4500` |
| Quantities | `u64` | 4 decimal places | `10000` = `1.0000` |
| Monetary exposure | `i128` | 8 decimal places | Full cross-currency precision |
| Percentages | `i32` | 2 decimal places | `1050` = `10.50%` |
| Basis points | `i32` | 0 decimal places | `250` = `250 bps` = `2.50%` |

### Scale Convention

All scaled integers use a **fixed, documented scale factor** per domain.
The scale factor is part of the type contract, not a runtime parameter.

```
value_human_readable = value_integer / (10 ^ scale)
```

---

## Rationale

### Why IEEE 754 Floating-Point Destroys Determinism

1. **Non-associativity**: `(a + b) + c â‰  a + (b + c)` in floating-point.
   Order of operations changes results. Replay with different execution order
   produces different state.

2. **Platform variance**: Different CPUs, SIMD instruction sets, and compilers
   produce different results for the same floating-point operations.

3. **Compiler optimization**: `-ffast-math` and similar flags reorder operations,
   silently changing results.

4. **Representation limits**: `0.1` cannot be exactly represented in binary
   floating-point. Repeated additions accumulate error.

5. **Silent drift**: Floating-point errors are small per operation but compound
   over millions of events in a replay session, eventually producing
   observably different state.

### Impact on AstraQuant OS

| Capability | Impact of Floats |
|-----------|-----------------|
| Deterministic replay | **Destroyed** â€” different float results per platform |
| State hash verification | **Destroyed** â€” hash mismatch from float drift |
| Snapshot recovery | **Destroyed** â€” recovered state diverges from original |
| Cross-system consistency | **Destroyed** â€” different hardware = different state |
| Audit trail integrity | **Destroyed** â€” cannot prove historical state |

---

## Enforcement

### Compile-Time

- Core event types use `u64`, `i64`, `i128` exclusively for financial values.
- No `f32` or `f64` fields in any `Serialize`/`Deserialize` struct that
  participates in state transitions.

### Code Review

- Any PR introducing `f32`, `f64`, or `float` in state-transition code
  must be rejected.

### Testing

- Integration tests verify that no floating-point operations affect
  state hash computation.
- Replay verification tests catch any float-induced drift.

---

## Exceptions

Floating-point is permitted ONLY in:

1. **Display/formatting** â€” converting fixed-point to human-readable output.
2. **Logging** â€” non-authoritative diagnostic output.
3. **External API boundaries** â€” converting from external float-based APIs
   to internal fixed-point representation (with explicit rounding rules).

Floating-point values MUST NEVER:
- Enter the event journal.
- Participate in state hash computation.
- Influence state transitions.
- Be stored in snapshots.

---

*This policy is permanent. It may not be relaxed without full architectural review.*

**AstraQuant OS v0.2.0** | Deterministic â€¢ Auditable â€¢ Replayable