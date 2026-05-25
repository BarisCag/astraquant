#!/usr/bin/env python3
"""
AstraQuant OS Core — Infrastructure Demonstration Suite.

Demonstrates deterministic financial infrastructure primitives:
1. Deterministic clock abstraction (NO time.time())
2. Schema-governed event creation with PayloadMetadata
3. Deterministic binary serialization & roundtrips
4. Cryptographic state hashing (blake3)
5. Event journal simulation with sequence ordering
6. Snapshot & replay verification workflow
7. Binary stability proof

Core Doctrine:
  - State is Sacred.
  - Replayability is law.
  - Infrastructure precedes intelligence.
  - The infrastructure layer NEVER owns time.
"""

from astra_core import (
    AstraEvent,
    EventType,
    PayloadEncoding,
    PayloadMetadata,
    SnapshotMetadata,
    py_serialize_event,
    py_deserialize_event,
    py_hash_event,
    py_hash_bytes,
    py_verify_hash_equality,
)


# =============================================================================
# Deterministic Clock — The infrastructure layer NEVER owns time.
# =============================================================================


class DeterministicClock:
    """Deterministic clock abstraction for replay-safe time injection.

    Replaces all wall-clock calls (time.time(), datetime.now(), etc.)
    with an explicit, controllable time source.

    This enables:
      - Deterministic replay (same inputs = same outputs)
      - Simulation (fast-forward, slow-motion)
      - Backtesting (historical time injection)
      - Chaos engineering (time anomalies)
      - Testing stability (no flaky clock-dependent tests)
    """

    def __init__(self, start_ns: int):
        """Initialize with a fixed starting timestamp in nanoseconds."""
        self._current_ns = start_ns

    def tick(self, delta_ns: int) -> int:
        """Advance time by delta_ns and return the new timestamp."""
        self._current_ns += delta_ns
        return self._current_ns

    def now(self) -> int:
        """Return current time without advancing."""
        return self._current_ns

    def __repr__(self) -> str:
        return f"DeterministicClock(current_ns={self._current_ns})"


# =============================================================================
# Example 1: Deterministic Event Creation with Schema Governance
# =============================================================================


def example_schema_governed_events():
    """Demonstrate mandatory PayloadMetadata on every event."""
    print("=" * 70)
    print("EXAMPLE 1: Schema-Governed Event Creation")
    print("=" * 70)

    clock = DeterministicClock(start_ns=1_700_000_000_000_000_000)

    # Every event MUST declare its encoding and schema version.
    # This is mandatory — not optional. Schema governance starts at creation.

    # Raw bytes event (schema_version 0 = unstructured)
    raw_event = AstraEvent(
        timestamp_ns=clock.tick(1_000_000),
        sequence_id=1,
        event_type=EventType.MarketTick,
        payload=b"\x00\x04\xD2",  # fixed-point price: 1234 (no floats!)
        payload_metadata=PayloadMetadata.raw(),
    )

    # JSON-encoded event with explicit schema version
    json_event = AstraEvent(
        timestamp_ns=clock.tick(1_000_000),
        sequence_id=2,
        event_type=EventType.StateSnapshot,
        payload=b'{"position_lots":100,"exposure_bps":500}',
        payload_metadata=PayloadMetadata(PayloadEncoding.Json, 1),
    )

    # Bincode-encoded event
    bincode_event = AstraEvent(
        timestamp_ns=clock.tick(1_000_000),
        sequence_id=3,
        event_type=EventType.OrderSubmitted,
        payload=b"\x01\x00\x00\x00\x64\x00\x00\x00",
        payload_metadata=PayloadMetadata(PayloadEncoding.Bincode, 2),
    )

    for event in [raw_event, json_event, bincode_event]:
        print(f"  {event}")
        print(f"    Encoding: {event.payload_metadata.encoding}")
        print(f"    Schema v: {event.payload_metadata.schema_version}")

    print("\n✓ All events created with mandatory schema governance")
    print()


# =============================================================================
# Example 2: Deterministic Serialization & Binary Stability
# =============================================================================


def example_binary_stability():
    """Prove deterministic serialization — identical events = identical bytes."""
    print("=" * 70)
    print("EXAMPLE 2: Binary Stability Proof")
    print("=" * 70)

    clock = DeterministicClock(start_ns=1_700_000_000_000_000_000)

    event = AstraEvent(
        timestamp_ns=clock.tick(1_000_000),
        sequence_id=1,
        event_type=EventType.OrderFilled,
        payload=b"\xDE\xAD\xBE\xEF",
        payload_metadata=PayloadMetadata(PayloadEncoding.RawBytes, 1),
    )

    # Serialize the same event three times
    b1 = py_serialize_event(event)
    b2 = py_serialize_event(event)
    b3 = py_serialize_event(event)

    # Byte-level equality
    assert b1 == b2 == b3, "DETERMINISM VIOLATION: serializations differ!"

    # Cryptographic equality (blake3)
    h1 = py_hash_bytes(b1)
    h2 = py_hash_bytes(b2)
    h3 = py_hash_bytes(b3)
    assert h1 == h2 == h3, "HASH VIOLATION: hashes differ!"

    print(f"  Serialized: {len(b1)} bytes")
    print(f"  Hash: {h1.hex()}")
    print(f"  3x serialization: IDENTICAL ✓")
    print(f"  3x blake3 hash:   IDENTICAL ✓")
    print("\n✓ Deterministic serialization cryptographically verified")
    print()


# =============================================================================
# Example 3: Cryptographic State Hashing
# =============================================================================


def example_state_hashing():
    """Demonstrate blake3 state hashing for replay verification."""
    print("=" * 70)
    print("EXAMPLE 3: Cryptographic State Hashing (blake3)")
    print("=" * 70)

    clock = DeterministicClock(start_ns=1_700_000_000_000_000_000)

    event = AstraEvent(
        timestamp_ns=clock.tick(1_000_000),
        sequence_id=1,
        event_type=EventType.MarketTick,
        payload=b"\x00\x04\xD2",
        payload_metadata=PayloadMetadata.raw(),
    )

    # Compute state hash
    event_hash = py_hash_event(event)
    print(f"  Event: {event}")
    print(f"  State Hash: 0x{event_hash.hex()}")

    # Verify hash survives serialization roundtrip
    serialized = py_serialize_event(event)
    recovered = py_deserialize_event(serialized)
    recovered_hash = py_hash_event(recovered)

    assert event_hash == recovered_hash, "REPLAY INVARIANT VIOLATED!"

    print(f"  Recovered Hash: 0x{recovered_hash.hex()}")
    print(f"  Hashes match after roundtrip: ✓")
    print("\n✓ State hash integrity verified through serialization")
    print()


# =============================================================================
# Example 4: Event Journal Simulation
# =============================================================================


def example_event_journal():
    """Simulate an append-only event journal with deterministic ordering."""
    print("=" * 70)
    print("EXAMPLE 4: Deterministic Event Journal")
    print("=" * 70)

    clock = DeterministicClock(start_ns=1_700_000_000_000_000_000)
    journal = []  # Append-only event journal (in-memory simulation)

    # Simulate a sequence of infrastructure events
    event_configs = [
        (EventType.MarketTick, b"\x00\x04\xD2"),       # price observation
        (EventType.OrderSubmitted, b"\x01\x00\x00"),    # order request
        (EventType.OrderFilled, b"\x01\x00\x64"),       # execution
        (EventType.RiskLimitBreached, b"\xFF"),          # threshold breach
        (EventType.StateSnapshot, b"\x00"),              # checkpoint
    ]

    for i, (event_type, payload) in enumerate(event_configs):
        event = AstraEvent(
            timestamp_ns=clock.tick(1_000_000),
            sequence_id=i + 1,
            event_type=event_type,
            payload=payload,
            payload_metadata=PayloadMetadata.raw(),
        )
        journal.append(py_serialize_event(event))

    print(f"  Journal: {len(journal)} events appended")

    # Verify journal integrity via replay
    for i, entry in enumerate(journal):
        recovered = py_deserialize_event(entry)
        assert recovered.sequence_id == i + 1, f"Sequence violation at {i}"
        print(f"    [{recovered.sequence_id}] {recovered.event_type} — {len(entry)} bytes")

    # Verify ordering invariant
    recovered_events = [py_deserialize_event(e) for e in journal]
    for i in range(1, len(recovered_events)):
        assert recovered_events[i].sequence_id > recovered_events[i - 1].sequence_id

    print("\n✓ Journal ordering invariant verified")
    print()


# =============================================================================
# Example 5: Snapshot & Replay Verification
# =============================================================================


def example_snapshot_replay():
    """Demonstrate snapshot creation and replay hash verification."""
    print("=" * 70)
    print("EXAMPLE 5: Snapshot & Replay Verification")
    print("=" * 70)

    clock = DeterministicClock(start_ns=1_700_000_000_000_000_000)

    # Simulate state accumulation through events
    events = []
    for i in range(5):
        event = AstraEvent(
            timestamp_ns=clock.tick(1_000_000),
            sequence_id=i + 1,
            event_type=EventType.MarketTick,
            payload=bytes([i * 10]),
            payload_metadata=PayloadMetadata.raw(),
        )
        events.append(event)

    # "State" is the concatenation of all event hashes (simplified model)
    state_bytes = b""
    for event in events:
        state_bytes += bytes(py_hash_event(event))

    # Create snapshot with blake3 hash of accumulated state
    state_hash = py_hash_bytes(state_bytes)
    snapshot = SnapshotMetadata(
        last_sequence_id=events[-1].sequence_id,
        state_hash=list(state_hash),
        subsystem_id="demo-engine",
    )

    print(f"  Events processed: {len(events)}")
    print(f"  Last sequence_id: {snapshot.last_sequence_id}")
    print(f"  State hash: 0x{bytes(snapshot.state_hash).hex()[:32]}...")
    print(f"  Subsystem: {snapshot.subsystem_id}")

    # === REPLAY: Reconstruct state from journal ===
    replay_state_bytes = b""
    for event in events:
        # Serialize → deserialize to simulate journal replay
        recovered = py_deserialize_event(py_serialize_event(event))
        replay_state_bytes += bytes(py_hash_event(recovered))

    replay_hash = py_hash_bytes(replay_state_bytes)

    print(f"\n  [REPLAY VERIFICATION]")
    print(f"  Original State Hash:  0x{bytes(state_hash).hex()[:32]}...")
    print(f"  Replayed State Hash:  0x{bytes(replay_hash).hex()[:32]}...")

    assert state_hash == replay_hash, "REPLAY DETERMINISM VIOLATED!"

    print(f"  Deterministic Replay: TRUE ✓")
    print("\n✓ Snapshot + journal replay produces IDENTICAL state hash")
    print()


# =============================================================================
# Example 6: Hash Verification Utility
# =============================================================================


def example_hash_verification():
    """Demonstrate hash equality verification for integrity checks."""
    print("=" * 70)
    print("EXAMPLE 6: Hash Integrity Verification")
    print("=" * 70)

    data_a = b"deterministic state representation"
    data_b = b"deterministic state representation"
    data_c = b"corrupted state representation"

    assert py_verify_hash_equality(data_a, data_b), "Same data must match"
    assert not py_verify_hash_equality(data_a, data_c), "Different data must differ"

    print(f"  Hash(A) == Hash(B): ✓ (identical data)")
    print(f"  Hash(A) != Hash(C): ✓ (corruption detected)")
    print("\n✓ Hash verification correctly detects integrity violations")
    print()


# =============================================================================
# Main
# =============================================================================


if __name__ == "__main__":
    print("\n" + "=" * 70)
    print("  AstraQuant OS Core — Infrastructure Demonstration Suite")
    print("  Doctrine: State is Sacred. Replayability is Law.")
    print("=" * 70 + "\n")

    example_schema_governed_events()
    example_binary_stability()
    example_state_hashing()
    example_event_journal()
    example_snapshot_replay()
    example_hash_verification()

    print("=" * 70)
    print("  ✓ All infrastructure demonstrations completed successfully.")
    print("  ✓ Zero wall-clock calls. Zero floating-point state.")
    print("  ✓ Deterministic replay cryptographically verified.")
    print("=" * 70)
