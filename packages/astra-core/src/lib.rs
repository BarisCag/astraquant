//! AstraQuant core: deterministic event-sourced trading kernel.

pub mod abi;
pub mod adapters;
pub mod audit;
pub mod clock;
pub mod cluster;
pub mod consensus;
pub mod dataset;
pub mod depth;
pub mod events;
pub mod exchange;
pub mod feed;
pub mod gas;
pub mod gateway;
pub mod hashing;
pub mod journal;
pub mod kernel;
pub mod ledger;
pub mod marketdata;
pub mod matching;
pub mod merkle;
pub mod metrics;
pub mod orchestrator;
pub mod orderbook;
pub mod package;
pub mod proof;
pub mod replay;
pub mod replication;
pub mod risk;
pub mod runtime;
pub mod sandbox;
pub mod serialization;
pub mod session;
pub mod simulation;
pub mod snapshot;
pub mod strategies;
pub mod strategy;
pub mod symbolic;
pub mod sync;
pub mod trades;
pub mod transport;
pub mod types;
pub mod verification;
pub mod vm;

pub use events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata, SnapshotMetadata};
pub use hashing::{hash_bytes, hash_to_hex, verify_hash_equality, DeterministicState};
pub use journal::EventJournal;
pub use kernel::AstraKernel;
pub use replay::{EventReducer, ReplayEngine, ReplayResult};
pub use serialization::{
    deserialize_canonical, deserialize_event, serialize_canonical, serialize_event,
    SerializationError,
};
pub use snapshot::{Snapshot, SnapshotManager};
