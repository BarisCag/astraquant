---
name: Feature request (Scope-Constrained)
about: Suggest an idea for AstraQuant OS that respects the research scope
title: "[FEATURE] "
labels: enhancement
assignees: ''

---

## 🎯 Feature Request

AstraQuant OS is a **deterministic quantitative systems research platform**. It is explicitly NOT a production hedge fund platform, nor a distributed network.

Before submitting this feature, please verify it respects the honest repository scope.

### Does this violate project scope?
- [ ] Does this introduce non-deterministic I/O (networking, wall-clock time, RNG) into `astra-core`? *(If yes, we will reject this.)*
- [ ] Does this add speculative architecture (e.g., distributed AI agents, blockchain consensus)? *(If yes, we will reject this.)*
- [ ] Can this be implemented cleanly within the existing `EventReducer` -> `state_hash` execution path?

### Description
A clear and concise description of what you want to happen. Explain how this furthers the goal of researching reproducible/replay-safe architecture.

### Implementation Concept
Describe how this integrates into the current event journal and `AstraKernel`. Are new `EventType` payloads required? How does it affect the cryptographic hash boundary?

### Alternatives Considered
A clear and concise description of any alternative solutions or features you've considered.
