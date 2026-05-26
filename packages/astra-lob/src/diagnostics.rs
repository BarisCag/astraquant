use crate::book::LimitOrderBook;
use crate::invariants::InvariantReport;
use crate::types::OrderEvent;
use astra_core::hashing::{hash_bytes, DeterministicState};
use astra_core::serialization::serialize_canonical;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayDiagnostics {
    pub total_orders: u64,
    pub total_trades: u64,
    pub total_cancels: u64,
    pub total_rejections: u64,
    pub total_partial_fills: u64,
    pub total_full_fills: u64,
    pub total_integrity_violations: u64,
    pub peak_bid_depth: u64,
    pub peak_ask_depth: u64,
    pub max_queue_length: u64,
}

impl DeterministicState for ReplayDiagnostics {
    fn state_hash(&self) -> [u8; 32] {
        let bytes =
            serialize_canonical(self).expect("ReplayDiagnostics canonical serialization failed");
        hash_bytes(&bytes)
    }
}

impl ReplayDiagnostics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ingest_events(&mut self, events: &[OrderEvent]) {
        for event in events {
            match event {
                OrderEvent::Accepted(_) => {
                    self.total_orders += 1;
                }
                OrderEvent::Rejected { .. } => {
                    self.total_rejections += 1;
                }
                OrderEvent::TradeExecuted(_) => {
                    self.total_trades += 1;
                    // Simplistic heuristic since we don't track full/partial in the event:
                    // In a highly accurate system, we'd add `is_full_fill` to TradeExecution.
                    // For now, we increment partial fills as a baseline proxy for execution volume.
                    self.total_partial_fills += 1;
                }
                OrderEvent::Cancelled { .. } => {
                    self.total_cancels += 1;
                }
                OrderEvent::Modified { .. } => {}
            }
        }
    }

    pub fn update_depth_metrics(&mut self, book: &LimitOrderBook) {
        let bid_depth = book.bids.len() as u64;
        let ask_depth = book.asks.len() as u64;

        if bid_depth > self.peak_bid_depth {
            self.peak_bid_depth = bid_depth;
        }
        if ask_depth > self.peak_ask_depth {
            self.peak_ask_depth = ask_depth;
        }

        let max_bid_queue = book
            .bids
            .values()
            .map(|l| l.orders.len() as u64)
            .max()
            .unwrap_or(0);
        let max_ask_queue = book
            .asks
            .values()
            .map(|l| l.orders.len() as u64)
            .max()
            .unwrap_or(0);
        let max_queue = max_bid_queue.max(max_ask_queue);

        if max_queue > self.max_queue_length {
            self.max_queue_length = max_queue;
        }
    }

    pub fn record_integrity_report(&mut self, report: &InvariantReport) {
        self.total_integrity_violations += report.violations.len() as u64;
    }
}
