//! Counterfactual Engine
//!
//! Runs two isolated phantom kernels against the same crisis dataset:
//!   - Baseline:      unmodified replay
//!   - Intervention:  modified replay
//!
//! Available interventions:
//!   - CircuitBreakerHalt: Pauses all events for `duration` sequences.
//!   - LiquidityInjection: Injects a synthetic bid wall (-3% price, +10000 volume)
//!   - ShortSellingBan: Rejects (skips) any down-tick with volume > threshold.

use astra_core::events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata};
use astra_core::exchange::ExchangeRuntime;
use astra_core::hashing::{hash_to_hex, DeterministicState};
use astra_core::kernel::AstraKernel;
use astra_core::marketdata::MarketTick;
use astra_core::replay::EventReducer;
use astra_core::risk::create_default_risk_engine;
use astra_core::runtime::StrategyRuntime;
use astra_core::serialization::{deserialize_canonical, serialize_canonical};
use astra_core::types::{Money, Price, Quantity};
use serde::{Deserialize, Serialize};

use crate::dataset_format::CrisisDataset;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterventionType {
    CircuitBreakerHalt { duration: u64 },
    LiquidityInjection,
    ShortSellingBan { volume_threshold: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualDelta {
    pub dataset_name: String,
    pub intervention_name: String,
    pub intervention_point: u64,
    pub baseline_hash: String,
    pub intervention_hash: String,
    pub baseline_price_nadir_raw: i64,
    pub intervention_price_nadir_raw: i64,
    pub price_delta_raw: i64,
    pub cascade_events_prevented: u64,
    pub hashes_diverged: bool,
    pub recovery_speed_delta: i64,
}

fn make_kernel() -> AstraKernel {
    let limits = create_default_risk_engine(Money::new(1_000_000_000), Quantity::new(100_000));
    AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits)))
}

fn build_halt_event(at_seq: u64) -> AstraEvent {
    AstraEvent::new(
        at_seq,
        at_seq,
        EventType::CircuitBreakerTriggered,
        b"CIRCUIT_BREAKER_HALT".to_vec(),
        PayloadMetadata::new(PayloadEncoding::RawBytes, 1),
    )
}

fn build_liquidity_event(at_seq: u64, current_price_raw: i64) -> AstraEvent {
    // Inject 10,000 synthetic bid orders at -3% from current price
    let inject_price = current_price_raw - (current_price_raw * 3 / 100);
    let tick = MarketTick {
        symbol: "INJECT".to_string(),
        timestamp_ns: at_seq,
        bid_price: Price::new(inject_price),
        ask_price: Price::new(inject_price + 5000),
        bid_quantity: Quantity::new(10_000),
        ask_quantity: Quantity::new(0),
    };
    let payload = serialize_canonical(&tick).unwrap();
    AstraEvent::new(
        at_seq,
        at_seq,
        EventType::LiquidityFacilityActivated,
        payload,
        PayloadMetadata::new(PayloadEncoding::Bincode, 1),
    )
}

pub struct CounterfactualEngine;

impl CounterfactualEngine {
    pub fn run(
        dataset_name: &str,
        dataset: &CrisisDataset,
        intervention: InterventionType,
        intervention_seq: u64,
    ) -> CounterfactualDelta {
        let mut baseline = make_kernel();
        let mut int_kernel = make_kernel();

        let mut baseline_nadir: i64 = i64::MAX;
        let mut int_nadir: i64 = i64::MAX;
        let mut cascade_prevented: u64 = 0;
        let mut halted_until: u64 = 0;
        let mut prev_price: i64 = 0;

        let intervention_name = match intervention {
            InterventionType::CircuitBreakerHalt { .. } => "CircuitBreaker",
            InterventionType::LiquidityInjection => "LiquidityInjection",
            InterventionType::ShortSellingBan { .. } => "ShortSellingBan",
        };

        for event in &dataset.events {
            let seq = event.sequence_id;

            // --- Baseline Replay ---
            let _ = baseline.apply(event);
            if let Ok(tick) = deserialize_canonical::<MarketTick>(&event.payload) {
                if tick.bid_price.0 < baseline_nadir {
                    baseline_nadir = tick.bid_price.0;
                }
            }

            // --- Intervention Logic ---
            let mut skip_event = false;
            let mut current_price = prev_price;
            let mut current_vol = 0;

            if let Ok(tick) = deserialize_canonical::<MarketTick>(&event.payload) {
                current_price = tick.bid_price.0;
                current_vol = tick.bid_quantity.0;
            }

            match intervention {
                InterventionType::CircuitBreakerHalt { duration } => {
                    if seq == intervention_seq {
                        let halt = build_halt_event(seq);
                        let _ = int_kernel.apply(&halt);
                        halted_until = seq + duration;
                    }
                    if seq > intervention_seq && seq <= halted_until {
                        skip_event = true;
                        cascade_prevented += 1;
                    }
                }
                InterventionType::LiquidityInjection => {
                    if seq == intervention_seq {
                        let inj = build_liquidity_event(seq, current_price);
                        let _ = int_kernel.apply(&inj);
                    }
                }
                InterventionType::ShortSellingBan { volume_threshold } => {
                    if seq >= intervention_seq {
                        // Reject large sell-offs (down-ticks with large volume)
                        if current_price < prev_price && current_vol > volume_threshold {
                            skip_event = true;
                            cascade_prevented += 1;
                        }
                    }
                }
            }

            if !skip_event {
                let _ = int_kernel.apply(event);
                if current_price > 0 && current_price < int_nadir {
                    int_nadir = current_price;
                }
            }
            if current_price > 0 {
                prev_price = current_price;
            }
        }

        if int_nadir == i64::MAX {
            int_nadir = baseline_nadir;
        }

        let baseline_hash = hash_to_hex(&baseline.state_hash());
        let intervention_hash = hash_to_hex(&int_kernel.state_hash());

        CounterfactualDelta {
            dataset_name: dataset_name.to_string(),
            intervention_name: intervention_name.to_string(),
            intervention_point: intervention_seq,
            hashes_diverged: baseline_hash != intervention_hash,
            price_delta_raw: int_nadir - baseline_nadir,
            baseline_price_nadir_raw: baseline_nadir,
            intervention_price_nadir_raw: int_nadir,
            cascade_events_prevented: cascade_prevented,
            recovery_speed_delta: 0, // Simplified for Phase 13A
            baseline_hash,
            intervention_hash,
        }
    }
}

pub struct BehavioralCounterfactualEngine;

impl BehavioralCounterfactualEngine {
    pub fn run(
        dataset_name: &str,
        dataset: &CrisisDataset,
        intervention: InterventionType,
        intervention_seq: u64,
        seed: astra_core::events::BehavioralSeed,
    ) -> CounterfactualDelta {
        use astra_agents::behavioral::{
            evaluate_anchor, evaluate_herding, evaluate_liquidity_withdrawal, evaluate_prospect,
            evaluate_salience, BehavioralAgentEnvironment,
        };

        let mut int_kernel = make_kernel();
        let mut int_nadir: i64 = i64::MAX;
        let mut cascade_prevented: u64 = 0;
        let mut halted_until: u64 = 0;
        let mut prev_price: i64 = 0;

        let intervention_name = match intervention {
            InterventionType::CircuitBreakerHalt { .. } => "Behavioral_CircuitBreaker",
            InterventionType::LiquidityInjection => "Behavioral_LiquidityInjection",
            InterventionType::ShortSellingBan { .. } => "Behavioral_ShortSellingBan",
        };

        let initial_price = if let Some(first) = dataset.events.first() {
            if let Ok(tick) = deserialize_canonical::<MarketTick>(&first.payload) {
                tick.bid_price.0
            } else {
                100_000_000
            }
        } else {
            100_000_000
        };

        let mut env = BehavioralAgentEnvironment::new(seed, initial_price);

        for event in &dataset.events {
            let seq = event.sequence_id;

            // --- Intervention Logic ---
            let mut skip_event = false;
            let mut current_price = prev_price;
            let mut current_vol = 0;

            if let Ok(tick) = deserialize_canonical::<MarketTick>(&event.payload) {
                current_price = tick.bid_price.0;
                current_vol = tick.bid_quantity.0;
            }

            match intervention {
                InterventionType::CircuitBreakerHalt { duration } => {
                    if seq == intervention_seq {
                        let halt = build_halt_event(seq);
                        let _ = int_kernel.apply(&halt);
                        halted_until = seq + duration;
                    }
                    if seq > intervention_seq && seq <= halted_until {
                        skip_event = true;
                        cascade_prevented += 1;
                    }
                }
                InterventionType::LiquidityInjection => {
                    if seq == intervention_seq {
                        let inj = build_liquidity_event(seq, current_price);
                        let _ = int_kernel.apply(&inj);
                    }
                }
                InterventionType::ShortSellingBan { volume_threshold } => {
                    if seq >= intervention_seq {
                        // Reject large sell-offs
                        if current_price < prev_price && current_vol > volume_threshold {
                            skip_event = true;
                            cascade_prevented += 1;
                        }
                    }
                }
            }

            if !skip_event {
                let _ = int_kernel.apply(event);

                // --- Behavioral Ecology Impact ---
                env.update_market(current_price);
                let i1 = evaluate_herding(&mut env);
                let i2 = evaluate_prospect(&mut env);
                let i3 = evaluate_anchor(&mut env);
                let i4 = evaluate_salience(&mut env);
                let i5 = evaluate_liquidity_withdrawal(&mut env);

                let total_sell_intent = [i1, i2, i3, i4, i5]
                    .iter()
                    .flat_map(|d| d.intents.iter())
                    .filter(|i| i.intent_type == "SELL")
                    .map(|i| i.size)
                    .sum::<u64>();

                // Price impact model
                let price_impact = (total_sell_intent / 100) as i64;
                current_price -= price_impact;

                if current_price > 0 && current_price < int_nadir {
                    int_nadir = current_price;
                }
            }
            if current_price > 0 {
                prev_price = current_price;
            }
        }

        let intervention_hash = hash_to_hex(&int_kernel.state_hash());

        CounterfactualDelta {
            dataset_name: dataset_name.to_string(),
            intervention_name: intervention_name.to_string(),
            intervention_point: intervention_seq,
            hashes_diverged: true,
            price_delta_raw: 0, // Needs baseline comparison context
            baseline_price_nadir_raw: 0,
            intervention_price_nadir_raw: int_nadir,
            cascade_events_prevented: cascade_prevented,
            recovery_speed_delta: 0,
            baseline_hash: String::new(),
            intervention_hash,
        }
    }
}
