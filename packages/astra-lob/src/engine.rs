use crate::book::LimitOrderBook;
use crate::diagnostics::ReplayDiagnostics;
use crate::types::Order;
use astra_core::events::{AstraEvent, EventType};
use astra_core::hashing::{hash_bytes, DeterministicState};
use astra_core::replay::EventReducer;
use astra_core::serialization::{deserialize_canonical, serialize_canonical};
use astra_core::types::Quantity;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CancelOrderPayload {
    pub order_id: u64,
    pub symbol: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModifyOrderPayload {
    pub order_id: u64,
    pub symbol: String,
    pub new_quantity: Quantity,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatchingEngine {
    pub books: BTreeMap<String, LimitOrderBook>,
    pub last_applied_sequence_id: Option<u64>,
    pub engine_sequence_id: u64,
    pub diagnostics: ReplayDiagnostics,
}

#[derive(Debug)]
pub enum EngineError {
    DeserializationFailed(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeserializationFailed(e) => write!(f, "Deserialization failed: {}", e),
        }
    }
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            books: BTreeMap::new(),
            last_applied_sequence_id: None,
            engine_sequence_id: 0,
            diagnostics: ReplayDiagnostics::new(),
        }
    }

    pub fn get_or_create_book(&mut self, symbol: &str) -> &mut LimitOrderBook {
        self.books
            .entry(symbol.to_string())
            .or_insert_with(|| LimitOrderBook::new(symbol.to_string()))
    }
}

impl Default for MatchingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DeterministicState for MatchingEngine {
    fn state_hash(&self) -> [u8; 32] {
        // Serializing the entire BTreeMap of LimitOrderBooks is fully deterministic
        let bytes =
            serialize_canonical(self).expect("MatchingEngine canonical serialization failed");
        hash_bytes(&bytes)
    }
}

impl EventReducer for MatchingEngine {
    type Error = EngineError;

    fn apply(&mut self, event: &AstraEvent) -> Result<(), Self::Error> {
        self.last_applied_sequence_id = Some(event.sequence_id);

        match event.event_type {
            EventType::OrderSubmitted => {
                let order: Order = deserialize_canonical(&event.payload)
                    .map_err(|e| EngineError::DeserializationFailed(e.to_string()))?;
                let symbol = order.symbol.clone();
                let book = self.books.entry(symbol.clone()).or_insert_with(|| LimitOrderBook::new(symbol));
                let events = book.submit(order);
                self.diagnostics.ingest_events(&events);
                self.diagnostics.update_depth_metrics(book);
                let report = book.validate_invariants();
                self.diagnostics.record_integrity_report(&report);
                self.engine_sequence_id += events.len() as u64;
            }
            EventType::LimitOrderCancelled => {
                let payload: CancelOrderPayload = deserialize_canonical(&event.payload)
                    .map_err(|e| EngineError::DeserializationFailed(e.to_string()))?;
                if let Some(book) = self.books.get_mut(&payload.symbol) {
                    let events = book.cancel(payload.order_id);
                    self.diagnostics.ingest_events(&events);
                    self.diagnostics.update_depth_metrics(book);
                    let report = book.validate_invariants();
                    self.diagnostics.record_integrity_report(&report);
                    self.engine_sequence_id += events.len() as u64;
                }
            }
            EventType::OperatorAction => {
                if let Ok(payload) = deserialize_canonical::<ModifyOrderPayload>(&event.payload) {
                    if let Some(book) = self.books.get_mut(&payload.symbol) {
                        let events = book.modify(payload.order_id, payload.new_quantity);
                        self.diagnostics.ingest_events(&events);
                        self.diagnostics.update_depth_metrics(book);
                        let report = book.validate_invariants();
                        self.diagnostics.record_integrity_report(&report);
                        self.engine_sequence_id += events.len() as u64;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn last_applied_sequence_id(&self) -> Option<u64> {
        self.last_applied_sequence_id
    }
}
