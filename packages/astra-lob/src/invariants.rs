use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntegrityViolation {
    CrossedBook,
    NegativeQuantity {
        order_id: u64,
        quantity: i64, // i64 allows negative quantities representation here
    },
    ZeroQuantityOrder {
        order_id: u64,
    },
    InvalidQueueOrdering {
        order_id: u64,
    },
    InvalidOrderIndex {
        order_id: u64,
    },
    InvalidBestBidAsk,
    InvalidSequenceOrdering,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvariantReport {
    pub violations: Vec<IntegrityViolation>,
}

impl InvariantReport {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
        }
    }

    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

impl Default for InvariantReport {
    fn default() -> Self {
        Self::new()
    }
}
