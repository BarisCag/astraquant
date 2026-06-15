use crate::types::{RiskViolation, TraderExposure, TraderRiskProfile};
use crate::velocity::VelocityWindow;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskEngine {
    pub trader_profiles: BTreeMap<u64, TraderRiskProfile>,
    pub trader_exposures: BTreeMap<u64, TraderExposure>,
    pub velocity_windows: BTreeMap<u64, VelocityWindow>,
    pub engine_sequence_id: u64,
}

impl Default for RiskEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskEngine {
    pub fn new() -> Self {
        Self {
            trader_profiles: BTreeMap::new(),
            trader_exposures: BTreeMap::new(),
            velocity_windows: BTreeMap::new(),
            engine_sequence_id: 0,
        }
    }

    pub fn register_trader(&mut self, profile: TraderRiskProfile) {
        let trader_id = profile.trader_id;
        self.trader_profiles.insert(trader_id, profile);
        self.trader_exposures
            .entry(trader_id)
            .or_insert(TraderExposure {
                trader_id,
                gross_exposure: 0,
                net_exposure: 0,
                realized_pnl: 0,
                open_orders: 0,
            });
        // Initialize velocity window with a default size if not present.
        // We can use 10,000 sequence ticks as a sensible default window for research.
        self.velocity_windows
            .entry(trader_id)
            .or_insert_with(|| VelocityWindow::new(10_000));
    }

    pub fn increment_sequence(&mut self) {
        self.engine_sequence_id += 1;
    }

    pub fn validate_order(
        &mut self,
        trader_id: u64,
        quantity: u64,
        notional: i64,
    ) -> Result<(), RiskViolation> {
        let profile = self
            .trader_profiles
            .get(&trader_id)
            .ok_or(RiskViolation::InvalidOrder)?;
        let exposure = self
            .trader_exposures
            .get_mut(&trader_id)
            .ok_or(RiskViolation::InvalidOrder)?;
        let velocity = self
            .velocity_windows
            .get_mut(&trader_id)
            .ok_or(RiskViolation::InvalidOrder)?;

        if quantity > profile.max_order_quantity {
            return Err(RiskViolation::MaxOrderQuantityExceeded);
        }

        velocity.evict_old_events(self.engine_sequence_id);
        if velocity.active_count() >= profile.max_order_velocity {
            return Err(RiskViolation::MaxOrderVelocityExceeded);
        }

        if exposure.realized_pnl < -profile.max_drawdown {
            return Err(RiskViolation::MaxDrawdownExceeded);
        }

        if exposure.gross_exposure.saturating_add(notional) > profile.max_position_notional {
            return Err(RiskViolation::MaxPositionExceeded);
        }

        // Track open order execution
        velocity.record_event(self.engine_sequence_id);
        exposure.open_orders += 1;

        // Note: For deterministic risk, we aggressively attribute gross exposure immediately.
        // Once filled or cancelled, we adjust appropriately.
        exposure.gross_exposure = exposure.gross_exposure.saturating_add(notional);

        Ok(())
    }

    pub fn apply_fill(
        &mut self,
        trader_id: u64,
        gross_change: i64,
        net_change: i64,
        realized_pnl: i64,
    ) {
        if let Some(exposure) = self.trader_exposures.get_mut(&trader_id) {
            // Net change represents the true executed directional notional
            exposure.net_exposure = exposure.net_exposure.saturating_add(net_change);
            // Gross change ensures our running limit accounts for open vs executed state
            exposure.gross_exposure = exposure.gross_exposure.saturating_add(gross_change);
            exposure.realized_pnl = exposure.realized_pnl.saturating_add(realized_pnl);
        }
    }

    pub fn apply_cancel(&mut self, trader_id: u64, notional_freed: i64) {
        if let Some(exposure) = self.trader_exposures.get_mut(&trader_id) {
            exposure.open_orders = exposure.open_orders.saturating_sub(1);
            exposure.gross_exposure = exposure.gross_exposure.saturating_sub(notional_freed);
        }
    }
}
