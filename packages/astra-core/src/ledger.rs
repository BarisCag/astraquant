use crate::events::AstraEvent;
use crate::hashing::{hash_bytes, DeterministicState};
use crate::replay::EventReducer;
use crate::serialization::{deserialize_canonical, serialize_canonical};
use crate::trades::Trade;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TradeLedger {
    pub trades: BTreeMap<u64, Trade>,
    pub last_applied_sequence_id: Option<u64>,
}

#[derive(Debug)]
pub enum LedgerError {
    DeserializationFailed(String),
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeserializationFailed(e) => write!(f, "Ledger deserialization failed: {}", e),
        }
    }
}

impl Default for TradeLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl TradeLedger {
    pub fn new() -> Self {
        Self {
            trades: BTreeMap::new(),
            last_applied_sequence_id: None,
        }
    }
}

impl DeterministicState for TradeLedger {
    fn state_hash(&self) -> [u8; 32] {
        let bytes = serialize_canonical(self).expect("TradeLedger serialization failed");
        hash_bytes(&bytes)
    }
}

impl EventReducer for TradeLedger {
    type Error = LedgerError;

    fn apply(&mut self, event: &AstraEvent) -> Result<(), Self::Error> {
        if event.event_type == crate::events::EventType::TradeSettled {
            if let Ok(trade) = deserialize_canonical::<Trade>(&event.payload) {
                self.trades.insert(trade.trade_id.0, trade);
            }
        }

        self.last_applied_sequence_id = Some(event.sequence_id);
        Ok(())
    }

    fn last_applied_sequence_id(&self) -> Option<u64> {
        self.last_applied_sequence_id
    }
}
