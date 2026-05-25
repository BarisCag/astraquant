//! Configurable risk limits enforced during exchange event application.

use crate::events::{AstraEvent, EventType};
use crate::portfolio::Portfolio;
use crate::replay::EventReducer;
use crate::types::{Money, Price, Quantity};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskLimits {
    pub max_position_notional: Money,
    pub max_order_quantity: Quantity,
    pub breach_count: u64,
    pub last_applied_sequence_id: Option<u64>,
}

impl RiskLimits {
    pub fn new(max_position_notional: Money, max_order_quantity: Quantity) -> Self {
        Self {
            max_position_notional,
            max_order_quantity,
            breach_count: 0,
            last_applied_sequence_id: None,
        }
    }

    pub fn validate_order(
        &self,
        portfolio: &Portfolio,
        symbol: &str,
        quantity: Quantity,
        price: Price,
    ) -> Result<(), String> {
        if quantity.0 > self.max_order_quantity.0 {
            return Err(format!(
                "order quantity {} exceeds max {}",
                quantity.0, self.max_order_quantity.0
            ));
        }

        let order_notional = (price.0 as i128).saturating_mul(quantity.0 as i128).abs();

        let existing_notional = portfolio
            .positions
            .get(symbol)
            .map(|p| {
                (p.average_entry_price.0 as i128)
                    .saturating_mul(p.quantity.unsigned_abs() as i128)
                    .abs()
            })
            .unwrap_or(0);

        if existing_notional.saturating_add(order_notional) > self.max_position_notional.0 {
            return Err(format!(
                "position notional would exceed limit {} for {}",
                self.max_position_notional.0, symbol
            ));
        }

        Ok(())
    }
}

impl EventReducer for RiskLimits {
    type Error = String;

    fn apply(&mut self, event: &AstraEvent) -> Result<(), Self::Error> {
        if event.event_type == EventType::RiskLimitBreached {
            self.breach_count = self.breach_count.saturating_add(1);
        }
        self.last_applied_sequence_id = Some(event.sequence_id);
        Ok(())
    }

    fn last_applied_sequence_id(&self) -> Option<u64> {
        self.last_applied_sequence_id
    }
}

impl crate::hashing::DeterministicState for RiskLimits {
    fn state_hash(&self) -> [u8; 32] {
        let mut data = Vec::new();
        data.extend_from_slice(&self.max_position_notional.0.to_le_bytes());
        data.extend_from_slice(&self.max_order_quantity.0.to_le_bytes());
        data.extend_from_slice(&self.breach_count.to_le_bytes());
        crate::hashing::hash_bytes(&data)
    }
}
