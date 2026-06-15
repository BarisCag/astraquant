use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueuePosition {
    pub initial_ahead_quantity: u64,
    pub initial_behind_quantity: u64,
    pub queue_sequence: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueueState {
    pub cumulative_depleted_quantity: u64,
    pub cumulative_added_quantity: u64,
    pub next_queue_sequence: u64,
}

impl QueueState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_back(&mut self, quantity: u64) -> QueuePosition {
        let position = QueuePosition {
            initial_ahead_quantity: self.cumulative_added_quantity,
            initial_behind_quantity: 0,
            queue_sequence: self.next_queue_sequence,
        };
        self.cumulative_added_quantity += quantity;
        self.next_queue_sequence += 1;
        position
    }

    pub fn record_depletion(&mut self, quantity: u64) {
        self.cumulative_depleted_quantity += quantity;
    }

    pub fn current_ahead_quantity(&self, position: &QueuePosition) -> u64 {
        position
            .initial_ahead_quantity
            .saturating_sub(self.cumulative_depleted_quantity)
    }

    pub fn current_behind_quantity(
        &self,
        position: &QueuePosition,
        remaining_order_quantity: u64,
    ) -> u64 {
        // Total currently resting is: cumulative_added - cumulative_depleted
        let total_resting = self
            .cumulative_added_quantity
            .saturating_sub(self.cumulative_depleted_quantity);
        let ahead = self.current_ahead_quantity(position);
        total_resting
            .saturating_sub(ahead)
            .saturating_sub(remaining_order_quantity)
    }
}
