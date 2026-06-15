//! Deterministic market data ingestion, normalization, and replay.
//!
//! `astra-stream` is the I/O boundary for market data. It handles non-deterministic
//! operations (WebSocket connections, wall-clock timestamps) and normalizes them into
//! deterministic fixed-point structures for journaling via `astra-core::EventJournal`.
//!
//! # Architecture boundary
//!
//! All external I/O (tokio, sockets, system clock) is quarantined inside this crate.
//! `astra-core` remains free of async runtimes, networking, and floating-point parsing.
//! The only dependency direction is `astra-stream → astra-core`.
//!
//! # Memory model
//!
//! - Each symbol holds one open `EventJournal` via `JournalRotator`
//! - Multi-symbol ingestion uses bounded `tokio::sync::mpsc` channels (capacity 1024)
//! - No unbounded queues anywhere in the ingestion or replay paths

pub mod certification;
pub mod fidelity;
pub mod ingestion;
pub mod metrics;
pub mod normalization;
pub mod normalized;
pub mod reconstruction;
pub mod replay;
pub mod rotation;
