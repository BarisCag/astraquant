---
name: Bug report (Deterministic)
about: Create a report to help us improve AstraQuant OS
title: "[BUG] "
labels: bug
assignees: ''

---

## 🛑 Bug Report (Deterministic Core)

AstraQuant OS is a deterministic research platform. We expect all bugs in `astra-core` to be 100% reproducible given a starting state and an event journal payload.

### Description
A clear and concise description of what the bug is.

### Reproduction Steps
Please provide the exact cryptographic sequence to reproduce this error:
1. Expected `sequence_id` state hash: `[Hex String]`
2. `AstraEvent` payload: `[JSON or Hex Bytes]`
3. Observed `state_hash` vs Expected `state_hash`
4. Any panic output or ReplayEngine failures

### Environment
- OS:
- Rust Toolchain version:
- Hardware Architecture (e.g. x86_64, aarch64):

### Additional Context
Any other context about the problem here. Does this occur during Live ingestion via `ExecutionGateway` or only during Journal Recovery?
