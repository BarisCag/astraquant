# Architecture Documentation

AstraQuant OS is structured into focused, composable packages enforcing mathematical determinism.

## astra-core
- **Purpose**: The foundational heart of the deterministic kernel, providing event journaling, cryptographic state hashing, and deterministic execution structures.
- **Key structs/traits**: `AstraKernel`, `EventJournal`, `ReplayEngine`, `DeterministicState`, `EventReducer`.
- **How it connects**: Depended upon by almost all other packages as the base mathematical model.
- **Critical invariants**: Enforces Blake3 hashing on all state transitions. Absolutely no floating-point arithmetic.

## astra-ops
- **Purpose**: The operations node runner that wires the kernel into a daemon, bridging non-deterministic IO into deterministic execution.
- **Key structs/traits**: `ExecutionGateway`, `OperationalTelemetry`.
- **How it connects**: Depends on `astra-core` and exposes it via HTTP API (Axum) and Prometheus metrics.
- **Critical invariants**: Ensures strict sequential event ingestion via the Gateway before the Kernel processes it.

## astra-research
- **Purpose**: A toolkit for defining crisis datasets, running phantom replays, generating analytical reports, and certifying deterministic traces.
- **Key structs/traits**: `PhantomRunner`, `DatasetReader`, `CrisisDataset`.
- **How it connects**: Uses `astra-core` and `astra-scenarios` to replay historical scenarios and generate outputs.
- **Critical invariants**: Ensures generated counterfactuals deterministically match expectations and maintains the `golden_hashes.json`.

## astra-research-api
- **Purpose**: An HTTP API exposing the `astra-research` capabilities to external interfaces for querying and dynamically injecting policy interventions.
- **Key structs/traits**: `ResearchServer`, `DatasetResponse`, `InterventionRequest`.
- **How it connects**: Wraps `astra-research` and `astra-core` features in a RESTful layer.
- **Critical invariants**: Must map HTTP requests to deterministic scenario runs identically every time.

## astra-rl
- **Purpose**: Provides Reinforcement Learning and sandbox capabilities (including WASM agent execution) to discover optimal policy interventions deterministically.
- **Key structs/traits**: `AgentSandbox`, `QTable`, `PolicyOptimizer`.
- **How it connects**: Integrates closely with `astra-core` state representations and `astra-policy`.
- **Critical invariants**: The Q-table and episode transitions must yield identical results across episodes when seeded identically.
