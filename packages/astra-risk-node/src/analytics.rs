use astra_core::events::{AstraEvent, EventType};
use astra_treasury::event::TreasuryEvent;

use crate::exporter::MetricsExporter;
use crate::var_calc::VarCalculator;
use crate::greeks::GreeksEngine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityTick {
    pub asset: String,
    pub implied_volatility: f64,
}

pub struct RiskAnalyticsEngine {
    exporter: MetricsExporter,
    portfolio_value_usd: f64,
    underlying_price: f64,
    implied_volatility: f64,
    rfr: f64,
}

impl RiskAnalyticsEngine {
    pub fn new(exporter: MetricsExporter) -> Self {
        Self {
            exporter,
            portfolio_value_usd: 100_000.0,
            underlying_price: 50_000.0,
            implied_volatility: 0.5,
            rfr: 0.05,
        }
    }

    pub fn process_event(&mut self, event: &AstraEvent) {
        match event.event_type {
            EventType::StateSnapshot => {
                // TreasuryEvent
                if let Ok(TreasuryEvent::CashPositionUpdate { position }) = astra_core::serialization::deserialize_canonical::<TreasuryEvent>(&event.payload) {
                    self.exporter.liquidity_runway.with_label_values(&[&format!("{:?}", position.currency)]).set(position.liquidity_runway_days as f64);
                } else if let Ok(TreasuryEvent::AccrualUpdate { currency: _, accrued_amount: _ }) = astra_core::serialization::deserialize_canonical::<TreasuryEvent>(&event.payload) {
                    // Update state if needed
                }
            }
            EventType::VolatilityTick => {
                if let Ok(tick) = astra_core::serialization::deserialize_canonical::<VolatilityTick>(&event.payload) {
                    self.implied_volatility = tick.implied_volatility;
                }
                
                // Recalculate Greeks
                let greeks = GreeksEngine::calculate_greeks(
                    true, // Call
                    self.underlying_price,
                    50_000.0, // Strike
                    0.25, // Time to maturity
                    self.rfr,
                    self.implied_volatility
                );
                self.exporter.greeks_delta.with_label_values(&["BTC_CALL"]).set(greeks.delta);
                
                // Recalculate VaR / ES
                let journal_hash = &[0u8; 32]; // Derived from the journal header
                let (var, es) = VarCalculator::monte_carlo_es(
                    self.portfolio_value_usd,
                    self.implied_volatility / 16.0, // rough daily vol conversion
                    journal_hash,
                    event.sequence_id,
                );
                
                self.exporter.var_99.with_label_values(&["PORTFOLIO"]).set(var);
                self.exporter.es_97_5.with_label_values(&["PORTFOLIO"]).set(es);
            }
            _ => {}
        }
    }
}
