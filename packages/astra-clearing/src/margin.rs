use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraderMarginProfile {
    pub initial_margin_ppm: u64,
    pub maintenance_margin_ppm: u64,
    pub liquidation_grace_sequences: u64,
    pub max_leverage_ppm: u64,
    pub collateral_haircut_ppm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CollateralAccount {
    pub trader_id: u64,
    pub total_collateral: i64,
    pub utilized_margin: u64,
    pub margin_health_ppm: i64, // (total_collateral * 1M) / utilized_margin
    pub active_margin_call: Option<MarginCall>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarginCall {
    pub call_sequence: u64,
    pub required_deposit: i64,
    pub deadline_sequence: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiquidationExecutionContext {
    pub trader_id: u64,
    pub symbol: String,
    pub target_sequence: u64,
    pub quantity: u64,
    pub is_buy: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MarginEngine {
    pub profiles: BTreeMap<u64, TraderMarginProfile>,
    pub accounts: BTreeMap<u64, CollateralAccount>,
    pub global_multiplier: u64,
}

impl MarginEngine {
    pub fn new() -> Self {
        Self {
            profiles: BTreeMap::new(),
            accounts: BTreeMap::new(),
            global_multiplier: 1_000_000,
        }
    }

    pub fn apply_emergency_haircut(&mut self, haircut_ppm: u64) {
        // Apply temporary emergency haircut modifier deterministically
        self.global_multiplier = (self.global_multiplier * haircut_ppm) / 1_000_000;
    }

    pub fn register_profile(&mut self, trader_id: u64, profile: TraderMarginProfile) {
        self.profiles.insert(trader_id, profile);
        self.accounts.entry(trader_id).or_insert(CollateralAccount {
            trader_id,
            total_collateral: 0,
            utilized_margin: 0,
            margin_health_ppm: 1_000_000,
            active_margin_call: None,
        });
    }

    pub fn update_collateral(&mut self, trader_id: u64, equity: i64, utilized_margin: u64) {
        if let Some(account) = self.accounts.get_mut(&trader_id) {
            let haircut = self
                .profiles
                .get(&trader_id)
                .map(|p| p.collateral_haircut_ppm)
                .unwrap_or(1_000_000);
            account.total_collateral = (equity * haircut as i64) / 1_000_000;
            account.utilized_margin = utilized_margin;

            if utilized_margin > 0 {
                account.margin_health_ppm =
                    (account.total_collateral * 1_000_000) / utilized_margin as i64;
            } else {
                account.margin_health_ppm = 1_000_000; // Safe
            }
        }
    }

    pub fn check_margin_health(
        &mut self,
        current_sequence: u64,
    ) -> Vec<LiquidationExecutionContext> {
        let mut liquidations = Vec::new();

        for (trader_id, account) in self.accounts.iter_mut() {
            if let Some(profile) = self.profiles.get(trader_id) {
                // If health breaches MM, issue margin call
                if account.margin_health_ppm < profile.maintenance_margin_ppm as i64 {
                    if account.active_margin_call.is_none() {
                        account.active_margin_call = Some(MarginCall {
                            call_sequence: current_sequence,
                            required_deposit: profile.initial_margin_ppm as i64
                                - account.total_collateral, // Just indicative
                            deadline_sequence: current_sequence
                                + profile.liquidation_grace_sequences,
                        });
                    }
                } else {
                    // Recovered
                    account.active_margin_call = None;
                }

                // If deadline passed, trigger liquidation
                if let Some(call) = &account.active_margin_call {
                    if current_sequence >= call.deadline_sequence {
                        // Will emit an event requiring liquidation
                        liquidations.push(LiquidationExecutionContext {
                            trader_id: *trader_id,
                            symbol: "".to_string(), // To be filled by ExchangeRuntime
                            target_sequence: current_sequence,
                            quantity: 0,
                            is_buy: false,
                        });
                        // Prevent repeated liquidations on the same sequence unless state updates
                        account.active_margin_call = None;
                    }
                }
            }
        }

        liquidations
    }
}
