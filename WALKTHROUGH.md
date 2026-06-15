# AstraQuant OS: Complete System Walkthrough (Phases 1 - 12)

This document is the definitive architectural and operational walkthrough of **AstraQuant OS**, detailing its evolution over 12 comprehensive engineering phases. It was designed to provide a deep understanding of the system's deterministic capabilities, operational boundaries, and cryptographic integrity.

---

## The Core Problem & Thesis
Quantitative finance relies on backtesting historical data. However, typical systems suffer from **operational non-determinism**—floating point drift, network latency, asynchronous wall-clock race conditions, and OS thread scheduling unpredictability. This means a strategy that worked perfectly in a backtest is never guaranteed to execute identically in production.

**AstraQuant OS** solves this by enforcing absolute determinism at the kernel level. The OS does not own a clock, it forbids floating point numbers, and it operates entirely on an append-only Event Journal. The live production environment is structurally identical to the historical simulation environment.

---

## Phases 1 & 2: The Deterministic Foundation
> **Core Doctrine:** The infrastructure NEVER owns time. State is sacred. Floating point arithmetic is forbidden. Blake3 is the only permitted hash.

We began by forging the fundamental primitives required for a deterministic universe:
- **Canonical Serialization (`serialization.rs`)**: Enforced a strict Bincode v2 boundary to ensure network bytes map identically to memory structures across all hardware architectures.
- **Event Journal (`journal.rs`)**: Built the `AstraEvent` schema and an append-only timeline. The journal acts as the absolute arbiter of time and sequence; no subsystem is allowed to generate its own timestamps.
- **Cryptographic Hashing (`hashing.rs`)**: Introduced the `DeterministicState` trait, forcing every subsystem to compute a Blake3 hash of its internal state, facilitating mathematically verifiable state transitions.

## Phase 3: Exchange Runtime
We transformed the basic event engine into a functioning financial layer:
- **Fixed-Point Primitives (`types.rs`)**: Eliminated floating-point drift by defining strict integer-backed `Price`, `Quantity`, and `Money` types (e.g., using 8-decimal implicit scaling).
- **Matching Engine (`matching.rs`)**: Implemented a classic BTreeMap-driven Limit Order Book utilizing strict Price-Time priority matching.
- **Ledger & Risk (`ledger.rs`, `risk.rs`)**: Built deterministic settlement tracking and limit exposure engines that reject incoming orders that mathematically exceed defined bounds.

## Phases 4 & 5: Strategy Runtime & Actor Kernel
We elevated the system from an exchange simulator into a programmable OS:
- **Execution DAG (`graph.rs`)**: Forced strategy logic to emit intent through a Directed Acyclic Graph, proving and isolating causality for every action.
- **Actor Isolation Kernel (`actor.rs`, `kernel.rs`)**: Introduced the `AstraKernel` which acts as the supreme authority over deterministic Actor mailboxes.
- **Scheduler & Supervisors (`scheduler.rs`, `supervisor.rs`)**: Detached scheduling from physical OS threads. Ticks are generated deterministically based purely on journal events, and crash domains can be reconstructed identically upon replay.

## Phases 6 & 7: Market Data, Simulation, & Gateways
We breached the barrier to the real world while protecting the core from its chaos:
- **Simulation Feed (`simulation.rs`)**: Designed a virtualized exchange clock and `HistoricalFeed` to ingest `.astra_ds` datasets for backtesting cycles that are 100% identical to live execution.
- **Execution Gateway (`gateway.rs`)**: The Border Wall. All non-deterministic inbound data (live WebSocket streams, user HTTP interactions) hits the Gateway, gets assigned a strict Sequence ID, is logged to the EventJournal, and only *then* interacts with the Kernel.
- **Normalized Depth (`depth.rs`)**: Consolidated raw external exchange data (e.g., Binance L2 limits) into canonical `DepthDelta` payloads.

## Phase 8: Distributed Consensus
We enabled AstraQuant OS to run globally across multiple physical servers:
- **Replication Buffer (`replication.rs`)**: Ensured journal events seamlessly synchronize across cluster nodes using a deterministic leader-follower model.
- **Verification Manifests (`verification.rs`)**: Nodes continuously exchange Blake3 state hashes. If Node A and Node B derive different hashes after consuming Sequence ID `N`, a consensus split is caught instantly, proving a divergence in deterministic execution.

## Phase 9: Secure WASM Sandboxing
We protected the OS from malicious or incompetent external strategy code:
- **Gas Metering (`gas.rs`)**: Enforced strict compute constraints on third-party logic. If a strategy infinitely loops, it exhausts its gas, deterministically aborts, and prevents cluster stalls without requiring an OS-level interrupt.
- **WASM VM (`sandbox.rs`)**: Checksum-verified `StrategyPackage` binaries execute within completely isolated memory contexts, talking to the kernel purely via a defined `HostCall` ABI.

## Phase 10: Formal Verification & Proofs
We elevated the system from "engineering trust" to "mathematical proof":
- **State Transition Proofs (`proof.rs`)**: Implemented mathematical logic verifying `STATE_t + EVENT_n = STATE_t+1`.
- **Merkle Roots (`merkle.rs`)**: Grouped cluster events into cryptographic Merkle trees for efficient, segmented auditing of historical slices.
- **Symbolic Fuzzing (`symbolic.rs`)**: Introduced aggressive twin-kernel fuzzing tools to hunt down any latent non-deterministic divergence paths before they enter production.

## Phase 11: Operational Control Plane
We finished the core architecture by adding an institutional exo-skeleton:
- **Non-Deterministic Quarantine (`astra-ops`)**: Stood up an entirely separate workspace (`daemon.rs`) using the `tokio` async runner to quarantine wall-clock behavior (sockets, ping/pongs, UI interactions) completely away from the sync, blocking `AstraKernel`.
- **Telemetry Bridge (`telemetry.rs`)**: Split observability into non-deterministic metrics (CPU, RAM, Socket Drops) and deterministic metrics (Gas consumed, DAG depth).
- **Replay Audit Engine (`audit.rs`)**: Provided the operational capability to spin up a "Phantom Kernel" on production journals to forensic-test an incident timeline in absolute mathematical isolation.
- **Operator Actions (`control.rs`)**: Made human operator actions (e.g. `KillStrategy`, `PauseSubsystem`) deterministic events that pass through the gateway, ensuring a journal replay perfectly mimics human intervention.

## Phase 12: Production Hardening & Deployment Reality
We encapsulated the entire system into robust, production-grade infrastructure:
- **Deployment Topology (`docker-compose.yml`)**: Containerized the `AstraDaemon` into a 3-node cluster, integrating a Prometheus and Grafana stack alongside persistent journal SSD volumes.
- **Observability Stack (`metrics_exporter.rs`)**: Exposed dynamic Grafana dashboards correlating deterministic variables (Gas, DAG Depth, Replay Hashes) with non-deterministic system bounds (WebSocket reconnects, network RTT).
- **Operational Lifecycle (`rotation.rs`)**: Engineered zero-downtime hot journal rotation, snapshot compaction, and gap-fill REST recoveries for external WebSocket disconnections.
- **Reliability Harness (`scripts/crash_loop_test.sh`)**: Built bash scripts and integration tests to brutally crash the OS via SIGKILL, validating bit-for-bit replay hash parity upon automatic restart.
- **CI/CD Verification (`.github/workflows`)**: Deployed deterministic Github Action gates that automatically fail Pull Requests if symbolic fuzzing diverges or historical dataset replays produce an unexpected hash.

---

## Final Architecture Diagram

```mermaid
graph TD;
    subgraph Non-Deterministic Quarantine (astra-ops)
        UI[Human Operator UI] -->|gRPC| gRPC_Server;
        LiveFeeds[Exchange Websockets] -->|TCP| Socket_Manager;
        gRPC_Server --> ControlPlane[Operator Commands];
        Socket_Manager --> ConnectionRecovery[Gap Recovery];
    end
    
    ControlPlane -- OperatorAction Payload --> Gateway;
    Socket_Manager -- Network Payload --> Gateway;
    
    subgraph Deterministic Environment (astra-core)
        Gateway[ExecutionGateway] --> Journal[(Append-Only EventJournal)];
        Journal --> Kernel[AstraKernel];
        
        subgraph AstraKernel
            Mailboxes --> DeterministicScheduler;
            DeterministicScheduler --> SupervisorTree;
            SupervisorTree --> WASM[Sandboxed VM Strategies];
            WASM --> MatchingEngine[Limit Order Books];
            MatchingEngine --> Portfolio[Risk & Ledger Settlement];
        end
        
        Kernel -.-> StateHash[Blake3 Merkle Root];
    end
```

## Conclusion
AstraQuant OS merges the rigid, bit-for-bit predictability of an embedded system with the complex, high-frequency needs of a modern financial trading environment. By meticulously quarantining non-determinism in the `astra-ops` layer and forcing every action through a strictly serialized, hashable `AstraEvent` loop in `astra-core`, the system achieves institutional-grade operational replayability.
