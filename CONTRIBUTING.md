# Contributing

Thank you for improving AstraQuant OS. This project prioritizes **deterministic correctness** and **documentation honesty** over feature breadth.

## Before you open a PR

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Guidelines

1. **Do not weaken replay guarantees** — changes to `journal.rs`, `replay.rs`, `snapshot.rs`, `hashing.rs`, and `serialization.rs` need extra scrutiny and tests.
2. **No floats in `astra-core`** — use fixed-point `Price`, `Quantity`, and `Money`.
3. **No theatrical tests** — assert state and hashes; avoid stdout-only success banners.
4. **Match docs to code** — if a subsystem is not wired, say so in README or remove the claim.
5. **Keep `astra-ops` non-deterministic** — wall-clock and network code stays out of `astra-core`.

## Commit messages

Use clear, professional subjects:

- `fix(replay): fail closed on verify_from hash mismatch`
- `test(exchange): assert ledger length after match`
- `docs: align deploy README with exported metrics`

## Scope we will decline

- Production-trading or HFT-readiness claims without benchmarks and audits
- Placeholder modules presented as operational
- Large architectural rewrites without a focused RFC in the issue

## Questions

Open a GitHub issue with the `question` label if replay semantics or reducer wiring are unclear.