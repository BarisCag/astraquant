use crate::venue::{VenueId, VenueState};
use astra_core::events::AstraEvent;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledOrder {
    pub target_sequence: u64,
    pub venue_id: VenueId,
    pub order_payload: AstraEvent,
    pub parent_order_id: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SmartOrderRouter {
    pub venues: BTreeMap<VenueId, VenueState>,
    // key: arrival sequence
    pub inflight_queue: BTreeMap<u64, Vec<ScheduledOrder>>,
    pub sequence_clock: u64,
}

impl SmartOrderRouter {
    pub fn new() -> Self {
        Self {
            venues: BTreeMap::new(),
            inflight_queue: BTreeMap::new(),
            sequence_clock: 0,
        }
    }

    pub fn add_venue(&mut self, state: VenueState) {
        self.venues.insert(state.venue_id, state);
    }

    pub fn route_order(&mut self, event: AstraEvent, parent_order_id: Option<u64>) {
        // Broadcast cancellations to all non-offline venues to ensure they are cancelled wherever they rest.
        if event.event_type == astra_core::events::EventType::LimitOrderCancelled {
            for venue in self.venues.values() {
                if venue.status != crate::venue::VenueStatus::Offline {
                    let target_sequence = self.sequence_clock + venue.latency_profile.ingress_delay_sequences;
                    let scheduled = ScheduledOrder {
                        target_sequence,
                        venue_id: venue.venue_id,
                        order_payload: event.clone(),
                        parent_order_id,
                    };
                    self.inflight_queue.entry(target_sequence).or_default().push(scheduled);
                }
            }
            return;
        }

        // For new orders, find the best venue: Active, then evaluate economics.
        let mut best_venue = None;
        for venue in self.venues.values() {
            if venue.status == crate::venue::VenueStatus::Offline || venue.status == crate::venue::VenueStatus::Paused || venue.status == crate::venue::VenueStatus::RejectOnly {
                continue;
            }
            // Deterministic Failover Rule:
            // lowest fee cost -> lowest latency -> lowest VenueId -> earliest venue sequence
            // For now, assume identical economics (fee cost = 0 for this simplified selection)
            let fee_cost = venue.fee_model.taker_fee_bps;
            let latency = venue.latency_profile.ingress_delay_sequences;
            let venue_id = venue.venue_id;
            let sequence = venue.venue_sequence_id;

            let current_score = (fee_cost, latency, venue_id, sequence);

            best_venue = match best_venue {
                None => Some((current_score, venue.venue_id)),
                Some((best_score, best_id)) => {
                    if current_score < best_score {
                        Some((current_score, venue.venue_id))
                    } else {
                        Some((best_score, best_id))
                    }
                }
            };
        }

        if let Some((_, best_id)) = best_venue {
            let venue = self.venues.get(&best_id).unwrap();
            let target_sequence = self.sequence_clock + venue.latency_profile.ingress_delay_sequences;
            let scheduled = ScheduledOrder {
                target_sequence,
                venue_id: venue.venue_id,
                order_payload: event,
                parent_order_id,
            };
            self.inflight_queue.entry(target_sequence).or_default().push(scheduled);
        }
    }

    pub fn advance_clock(&mut self, sequence_id: u64) -> Vec<ScheduledOrder> {
        self.sequence_clock = sequence_id;
        let mut arrived = Vec::new();
        // Extract all orders where target_sequence <= sequence_clock
        let mut to_remove = Vec::new();
        for (seq, orders) in self.inflight_queue.range(..=self.sequence_clock) {
            arrived.extend(orders.iter().cloned());
            to_remove.push(*seq);
        }
        for seq in to_remove {
            self.inflight_queue.remove(&seq);
        }
        arrived
    }

    pub fn state_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.sequence_clock.to_le_bytes());
        for (venue_id, venue) in &self.venues {
            hasher.update(&[venue_id.0]);
            hasher.update(&venue.state_hash());
        }
        for (seq, orders) in &self.inflight_queue {
            hasher.update(&seq.to_le_bytes());
            for order in orders {
                hasher.update(&order.target_sequence.to_le_bytes());
                hasher.update(&[order.venue_id.0]);
                // Simplified hash for inflight events
                hasher.update(&order.order_payload.sequence_id.to_le_bytes());
            }
        }
        *hasher.finalize().as_bytes()
    }
}
