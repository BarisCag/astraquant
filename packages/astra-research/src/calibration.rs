//! Behavioral Calibration Engine
//!
//! Sweeps behavioral parameters to best fit historical cascade depth.

use crate::dataset_format::CrisisDataset;
use astra_agents::behavioral::{
    evaluate_anchor, evaluate_herding, evaluate_liquidity_withdrawal, evaluate_prospect,
    evaluate_salience, BehavioralAgentEnvironment,
};
use astra_core::events::BehavioralSeed;
use astra_core::marketdata::MarketTick;
use astra_core::serialization::deserialize_canonical;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct CalibrationResult {
    pub crisis: String,
    pub best_herding_factor: f64,
    pub best_loss_aversion: f64,
    pub fit_score: f64,
    pub cascade_depth_error_pct: f64,
}

pub struct CalibrationEngine;

impl CalibrationEngine {
    pub fn run(dataset_name: &str, dataset: &CrisisDataset) -> CalibrationResult {
        let herding_factors = vec![0.1, 0.3, 0.5, 0.7, 0.9];
        let loss_aversions = vec![1.5, 2.0, 2.5];

        let historical_nadir = Self::get_historical_nadir(dataset);

        let mut best_result = CalibrationResult {
            crisis: dataset_name.to_string(),
            best_herding_factor: 0.0,
            best_loss_aversion: 0.0,
            fit_score: f64::MAX,
            cascade_depth_error_pct: 100.0,
        };

        let initial_price = Self::get_initial_price(dataset);

        for &herding in &herding_factors {
            for &loss_aversion in &loss_aversions {
                let seed = BehavioralSeed::new(herding, loss_aversion, 0.5, 0.8, 42);
                let mut env = BehavioralAgentEnvironment::new(seed, initial_price);

                let mut sim_nadir = initial_price;
                let mut current_price = initial_price;

                for event in &dataset.events {
                    if let Ok(tick) = deserialize_canonical::<MarketTick>(&event.payload) {
                        current_price = tick.bid_price.0;
                    }

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

                    // Simulate simple price impact (100 units drop per 10k sell volume)
                    let price_impact = (total_sell_intent / 100) as i64;
                    current_price -= price_impact;

                    if current_price < sim_nadir {
                        sim_nadir = current_price;
                    }
                }

                let depth_error = (sim_nadir - historical_nadir).abs() as f64;
                let depth_error_pct = depth_error / (initial_price as f64) * 100.0;

                if depth_error < best_result.fit_score {
                    best_result.fit_score = depth_error;
                    best_result.best_herding_factor = herding;
                    best_result.best_loss_aversion = loss_aversion;
                    best_result.cascade_depth_error_pct = depth_error_pct;
                }
            }
        }

        best_result
    }

    fn get_historical_nadir(dataset: &CrisisDataset) -> i64 {
        let mut nadir = i64::MAX;
        for event in &dataset.events {
            if let Ok(tick) = deserialize_canonical::<MarketTick>(&event.payload) {
                if tick.bid_price.0 < nadir {
                    nadir = tick.bid_price.0;
                }
            }
        }
        nadir
    }

    fn get_initial_price(dataset: &CrisisDataset) -> i64 {
        for event in &dataset.events {
            if let Ok(tick) = deserialize_canonical::<MarketTick>(&event.payload) {
                return tick.bid_price.0;
            }
        }
        100_000_000
    }
}
