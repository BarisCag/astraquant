# EVENT_JOURNAL_SPEC.md â€” Constitutional Law of AstraQuant OS

> This document is the constitutional foundation of the AstraQuant OS event system.
> All subsystems, engines, and services MUST comply with these specifications.
> Violations are treated as system corruption.

---

## 1. Append Semantics

Events are **append-only**. Once committed to the journal:

- Events are **NEVER** mutated.
- Events are **NEVER** deleted.
- Events are **NEVER** reordered after commitment.

The journal is a permanent, immutable record of all state transitions.

**Compaction:** The journal itself is never compacted. Snapshots enable efficient
replay by providing a starting checkpoint, but the full journal is retained
for auditability and forensic analysis.

---

## 2. Ordering Guarantees

### Global Monotonic Sequence IDs

```
sequence_id: u64
```

- A **globally monotonic** identifier assigned **exclusively** by the event journal.
- **No two committed events** may share the same `sequence_id`.
- **Ordering authority** belongs to the journal, not producers.
- Producers submit events; the journal assigns sequence IDs upon commitment.

### Ordering Invariants

- `sequence_id(N+1) > sequence_id(N)` for all consecutive committed events.
- **Gaps** indicate lost, filtered, or rejected events. Gaps are observable but not errors.
- **Duplicates** indicate corruption. Duplicate sequence IDs are always errors.
- `timestamp_ns` provides causal context but is NOT the ordering authority.
  Events with out-of-order timestamps but correct sequence IDs are valid.

---

## 3. Replay Guarantees

### The Fundamental Invariant

```
Identical Journal + Identical Initial State = Identical Final State Hash
```

This is not approximate. Not "same business meaning." **IDENTICAL HASH.**

### Replay Requirements

- All state transitions are derived exclusively from journal events.
- No external I/O during replay (network, filesystem, clock).
- No implicit randomness during replay.
- No floating-point arithmetic in state transitions.
- Deterministic ordering of all operations.

### Replay Modes

| Mode | Description |
|------|-------------|
| Full Replay | Replay from genesis (empty state + full journal) |
| Snapshot Replay | Replay from snapshot + subsequent journal entries |
| Verification Replay | Replay and compare final state hash against expected |

---

## 4. Snapshot Boundaries

### Snapshot Contract

A **StateSnapshot** event represents a fully self-contained, deterministic
reconstruction checkpoint for a bounded subsystem.

### Snapshot Requirements

| Requirement | Description |
|-------------|-------------|
| **Complete** | Contains entire subsystem state. No external dependencies for reconstruction. |
| **Replay-Portable** | Can be restored on any compatible system without additional context. |
| **Hash-Verifiable** | Includes a blake3 state hash that proves integrity. |
| **Sequenced** | Includes `last_sequence_id` â€” the sequence ID of the last event absorbed. |
| **Identified** | Includes `subsystem_id` â€” which subsystem produced this snapshot. |

### Snapshot Verification Invariant

```
Snapshot(S) + Replay(Journal[S.last_sequence_id + 1 .. N]) = State(N)
blake3(State(N)) MUST equal expected state hash
```

### SnapshotMetadata Structure

```rust
pub struct SnapshotMetadata {
    pub last_sequence_id: u64,   // Last event absorbed
    pub state_hash: [u8; 32],    // Blake3 hash of canonical state
    pub subsystem_id: String,    // Originating subsystem
}
```

---

## 5. Checksum Policy

### Hash Function: Blake3 ONLY

- **Blake3** is the sole permitted hash function in AstraQuant OS.
- No SHA-256. No MD5. No configurable hash selection.
- Deterministic infrastructure requires uniform cryptographic semantics.

### What Gets Hashed

| Subject | Hash Purpose |
|---------|-------------|
| Individual events | State identity via `DeterministicState::state_hash()` |
| Accumulated state | Snapshot integrity verification |
| Serialized bytes | Binary stability proof |
| Journal segments | Corruption detection |

### Hash Computation Rules

- All hashing uses **canonical serialized form** (see Â§11 Canonical Serialization).
- Hash inputs must be deterministic â€” no HashMap iteration, no platform-dependent behavior.
- `DeterministicState::state_hash()` is the standard interface.

---

## 6. Corruption Detection

### Detectable Corruption Signals

| Signal | Meaning |
|--------|---------|
| Duplicate `sequence_id` | Journal write corruption |
| `sequence_id` gap beyond threshold | Event loss |
| State hash mismatch after replay | Nondeterminism or data corruption |
| Payload deserialization failure | Schema mismatch or bit rot |
| Unknown `PayloadEncoding` value | Version incompatibility |
| Snapshot hash mismatch | Snapshot corruption |

### Response to Corruption

1. **HALT** affected subsystem immediately.
2. **LOG** corruption signal with full diagnostic context.
3. **NEVER** silently continue with corrupted state.
4. **RECOVER** from last valid snapshot + verified journal replay.

---

## 7. Recovery Semantics

### Recovery Procedure

1. Identify last valid snapshot for affected subsystem.
2. Verify snapshot integrity via `state_hash`.
3. Replay all journal events after `snapshot.last_sequence_id`.
4. Verify final state hash matches expected value.
5. If verification succeeds: resume normal operation.
6. If verification fails: escalate to operator intervention.

### Recovery Invariant

Recovery MUST produce a state that is **hash-identical** to the state
that existed before the failure. If it does not, the recovery has failed
and the system must not resume.

---

## 8. Event Versioning

### PayloadMetadata Governance

Every event carries mandatory `PayloadMetadata`:

```rust
pub struct PayloadMetadata {
    pub encoding: PayloadEncoding,   // Wire format (RawBytes, Json, Bincode, etc.)
    pub schema_version: u16,         // Schema revision number
}
```

### Schema Evolution Rules

| Action | Procedure |
|--------|-----------|
| Add field | Bump `schema_version`. New consumers handle new field; old consumers ignore it. |
| Remove field | Bump `schema_version`. Old consumers must still deserialize (field absent). |
| Change encoding | Bump `schema_version`. Document migration path. |
| Breaking change | Major version bump. Dual-write transition period required. |

### Version 0 Semantics

`schema_version: 0` with `PayloadEncoding::RawBytes` indicates unstructured
bytes with no schema governance. This is permitted for infrastructure signals
but discouraged for domain events.

---

## 9. Retention Policy

### Events

- Events are **permanent**. The journal is the system of record.
- No automatic deletion or TTL-based expiration.
- Archival to cold storage is permitted but the logical journal remains complete.

### Snapshots

- Snapshots enable efficient replay but do not replace the journal.
- Old snapshots may be pruned once newer, verified snapshots exist.
- At least one verified snapshot per subsystem must be retained.

---

## 10. Floating-Point Policy

### Absolute Prohibition

```
The AstraQuant OS event layer FORBIDS floating-point financial state.
```

All prices, quantities, and monetary values MUST use deterministic
fixed-point encoding:

| Domain | Type | Example |
|--------|------|---------|
| Prices | `i64` scaled integer | `1234500 = 123.4500` at scale=4 |
| Quantities | `u64` scaled integer | `10000 = 1.0000` at scale=4 |
| Monetary exposure | `i128` | Full cross-currency precision |
| Percentages | scaled integers | `1050 = 10.50%` at scale=2 |

### Rationale

IEEE 754 floating-point arithmetic is:
- **Non-associative**: `(a + b) + c â‰  a + (b + c)`
- **Platform-dependent**: SIMD, compiler optimizations, and FPU differences
- **Replay-destroying**: Silent precision drift across replay sessions

---

## 11. Canonical Serialization Policy

### What Constitutes Canonical Serialized State

All serialization in AstraQuant OS follows these rules:

| Rule | Specification |
|------|--------------|
| **Field ordering** | Struct fields serialize in **declaration order**. No reordering. |
| **Byte order** | **Little-endian** exclusively. |
| **Integer encoding** | **Fixed-width** encoding. No variable-length integers. |
| **String encoding** | **UTF-8** only. No normalization transforms during serialization. |
| **Collections** | **Ordered only**. `Vec`, `BTreeMap`, `BTreeSet`. **HashMap/HashSet FORBIDDEN** in serializable state. |
| **Optional values** | Explicit `Option<T>` serialization. Missing â‰  None. |
| **Configuration** | `bincode::config::standard().with_little_endian().with_fixed_int_encoding()` |

### Forbidden Nondeterministic Patterns

- `HashMap` / `HashSet` in any serializable type (iteration order is undefined)
- Platform-dependent types (`usize`, `isize`) in serialized state
- Floating-point values in financial state
- System clock reads during serialization
- Random number generation during serialization
- Locale-dependent string formatting

### Canonical Config (Bincode v2)

```rust
fn bincode_config() -> impl bincode::config::Config {
    config::standard()
        .with_little_endian()
        .with_fixed_int_encoding()
}
```

**This configuration MUST NOT be modified.** Changing it breaks all existing journal data.

---

## 12. Deterministic Clock Policy

### The infrastructure layer NEVER owns time.

- All timestamps (`timestamp_ns: u64`) are **externally injected**.
- No calls to `std::time::SystemTime::now()`, `time.time()`, `datetime.now()`, or equivalent.
- Clock sources are provided by the execution environment, not the infrastructure.

### Deterministic Clock Abstraction

```python
class DeterministicClock:
    def __init__(self, start_ns: int):
        self._current_ns = start_ns

    def tick(self, delta_ns: int) -> int:
        self._current_ns += delta_ns
        return self._current_ns

    def now(self) -> int:
        return self._current_ns
```

### Clock Injection Enables

| Capability | Description |
|-----------|-------------|
| Deterministic replay | Same clock inputs â†’ same state outputs |
| Backtesting | Historical timestamp injection |
| Simulation | Arbitrary time progression |
| Chaos engineering | Time anomaly injection |
| Test stability | No flaky clock-dependent tests |

---

*This specification is versioned alongside the codebase. Changes require architectural review.*

**AstraQuant OS v0.2.0** | Deterministic â€¢ Auditable â€¢ Replayable