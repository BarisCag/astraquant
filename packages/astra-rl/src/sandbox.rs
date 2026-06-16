use crate::state_space::StateVector;
use astra_core::events::AstraEvent;

pub struct RLSandbox {
    pub is_active: bool,
    pub current_state: Option<StateVector>,
    pub current_reward: f64,
    pub position: i64,
}

impl RLSandbox {
    pub fn new() -> Self {
        Self {
            is_active: false,
            current_state: None,
            current_reward: 0.0,
            position: 0,
        }
    }

    pub fn activate(&mut self) {
        self.is_active = true;
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn observe(&mut self, event: &AstraEvent) {
        if self.is_active {
            self.current_state = Some(crate::state_space::event_to_tensor(event, self.position));
        }
    }

    pub fn step(&mut self, action: i64, pnl: i64) -> (StateVector, f64, bool) {
        self.position += action;
        let reward = crate::reward::RewardFunction::Pnl.evaluate(pnl, 0);
        self.current_reward += reward;

        let state = self.current_state.clone().unwrap_or_else(|| StateVector {
            features: vec![0.0; 4],
        });
        (state, reward, false)
    }
}

impl Default for RLSandbox {
    fn default() -> Self {
        Self::new()
    }
}
