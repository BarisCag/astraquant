use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VelocityWindow {
    pub window_size: u64,
    pub events: VecDeque<u64>,
}

impl VelocityWindow {
    pub fn new(window_size: u64) -> Self {
        Self {
            window_size,
            events: VecDeque::new(),
        }
    }

    pub fn record_event(&mut self, current_sequence_id: u64) {
        self.events.push_back(current_sequence_id);
    }

    pub fn evict_old_events(&mut self, current_sequence_id: u64) {
        let cutoff = current_sequence_id.saturating_sub(self.window_size);
        while let Some(&seq) = self.events.front() {
            if seq < cutoff {
                self.events.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn active_count(&self) -> u64 {
        self.events.len() as u64
    }
}
