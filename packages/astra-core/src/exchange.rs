//! Exchange runtime: matching engine, portfolio, ledger, and risk limits.
//!
//! # Legacy Warning
//! This module represents the minimal legacy deterministic runtime.
//! The current authoritative orchestration layer is `astra-exchange::ExchangeRuntime`,
//! which integrates `astra-lob` and `astra-portfolio` advanced infrastructure.

use crate::events::{AstraEvent, EventType};
use crate::hashing::{hash_bytes, DeterministicState};
use crate::ledger::TradeLedger;
use crate::matching::MatchingEngine;
use crate::orderbook::{
    LimitOrderCancelledPayload, LimitOrderMatchedPayload, LimitOrderPlacedPayload, OrderSide,
};
use crate::replay::EventReducer;
use crate::serialization::deserialize_canonical;
use crate::trades::{Trade, TradeId};
use astra_portfolio::engine::PositionEngine;
use astra_risk::engine::RiskEngine;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExchangeRuntime {
    pub risk_engine: RiskEngine,
    pub markets: BTreeMap<String, MatchingEngine>,
    pub position_engine: PositionEngine,
    pub ledger: TradeLedger,
    pub last_applied_sequence_id: Option<u64>,
}

impl ExchangeRuntime {
    pub fn new(risk_engine: RiskEngine) -> Self {
        Self {
            risk_engine,
            markets: BTreeMap::new(),
            position_engine: PositionEngine::new(),
            ledger: TradeLedger::new(),
            last_applied_sequence_id: None,
        }
    }

    pub fn add_market(&mut self, symbol: String) {
        self.markets
            .entry(symbol.clone())
            .or_insert_with(|| MatchingEngine::new(symbol));
    }

    fn settle_match(
        &mut self,
        symbol: &str,
        timestamp_ns: u64,
        sequence_id: u64,
        match_index: u32,
        taker_side: OrderSide,
        match_payload: &LimitOrderMatchedPayload,
    ) {
        let trade_id = TradeId(
            sequence_id
                .saturating_mul(10_000)
                .saturating_add(match_index as u64),
        );
        let trade = Trade {
            trade_id,
            symbol: symbol.to_string(),
            taker_side,
            maker_order_id: match_payload.maker_order_id,
            taker_order_id: match_payload.taker_order_id,
            maker_trader_id: match_payload.maker_trader_id,
            taker_trader_id: match_payload.taker_trader_id,
            price: match_payload.match_price,
            quantity: match_payload.matched_quantity,
            timestamp_ns,
        };
        let is_taker_buy = trade.taker_side == OrderSide::Bid;
        // Update taker
        self.position_engine.apply_fill(
            trade.taker_trader_id,
            &trade.symbol,
            is_taker_buy,
            trade.quantity.0,
            trade.price.0,
        );
        // Update maker
        self.position_engine.apply_fill(
            trade.maker_trader_id,
            &trade.symbol,
            !is_taker_buy,
            trade.quantity.0,
            trade.price.0,
        );

        self.position_engine
            .update_mark_price(&trade.symbol, trade.price.0);

        self.ledger.trades.insert(trade.trade_id.0, trade);
    }

    fn apply_limit_order_placed(
        &mut self,
        event: &AstraEvent,
        payload: &LimitOrderPlacedPayload,
    ) -> Result<(), String> {
        self.risk_engine.increment_sequence();
        let notional = payload.price.0.saturating_mul(payload.quantity.0 as i64);
        if let Err(e) =
            self.risk_engine
                .validate_order(payload.trader_id, payload.quantity.0, notional)
        {
            return Err(format!("Risk Violation: {:?}", e));
        }

        if !self.markets.contains_key(&payload.symbol) {
            self.add_market(payload.symbol.clone());
        }

        let matches = {
            let engine = self
                .markets
                .get_mut(&payload.symbol)
                .expect("market exists after insert");
            engine.process_limit_order(payload)
        };

        for (i, m) in matches.iter().enumerate() {
            self.settle_match(
                &payload.symbol,
                event.timestamp_ns,
                event.sequence_id,
                i as u32,
                payload.side,
                m,
            );
        }

        if let Some(engine) = self.markets.get_mut(&payload.symbol) {
            engine.book.last_applied_sequence_id = Some(event.sequence_id);
        }
        Ok(())
    }
}

impl EventReducer for ExchangeRuntime {
    type Error = String;

    fn apply(&mut self, event: &AstraEvent) -> Result<(), Self::Error> {
        match event.event_type {
            EventType::LimitOrderPlaced | EventType::OrderSubmitted => {
                let payload: LimitOrderPlacedPayload =
                    deserialize_canonical(&event.payload).map_err(|e| e.to_string())?;
                self.apply_limit_order_placed(event, &payload)?;
            }
            EventType::LimitOrderCancelled => {
                let payload: LimitOrderCancelledPayload =
                    deserialize_canonical(&event.payload).map_err(|e| e.to_string())?;
                if let Some(engine) = self.markets.get_mut(&payload.symbol) {
                    engine.cancel_order(payload.order_id);
                    engine.book.last_applied_sequence_id = Some(event.sequence_id);
                }
            }
            EventType::LimitOrderMatched => {
                let payload: LimitOrderMatchedPayload =
                    deserialize_canonical(&event.payload).map_err(|e| e.to_string())?;
                for engine in self.markets.values_mut() {
                    if engine
                        .book
                        .order_index
                        .contains_key(&payload.maker_order_id)
                    {
                        engine
                            .book
                            .apply_matched(&payload)
                            .map_err(|e| e.to_string())?;
                        engine.book.last_applied_sequence_id = Some(event.sequence_id);
                        break;
                    }
                }
            }
            EventType::TradeSettled => {
                if let Ok(trade) = deserialize_canonical::<Trade>(&event.payload) {
                    let is_taker_buy = trade.taker_side == OrderSide::Bid;
                    self.position_engine.apply_fill(
                        trade.taker_trader_id,
                        &trade.symbol,
                        is_taker_buy,
                        trade.quantity.0,
                        trade.price.0,
                    );
                    self.position_engine.apply_fill(
                        trade.maker_trader_id,
                        &trade.symbol,
                        !is_taker_buy,
                        trade.quantity.0,
                        trade.price.0,
                    );
                    self.position_engine
                        .update_mark_price(&trade.symbol, trade.price.0);

                    self.ledger.trades.insert(trade.trade_id.0, trade);
                }
            }
            EventType::RiskLimitBreached => {
                // If we want to record an external breach to increment risk, we can.
            }
            _ => {}
        }

        self.last_applied_sequence_id = Some(event.sequence_id);
        Ok(())
    }

    fn last_applied_sequence_id(&self) -> Option<u64> {
        self.last_applied_sequence_id
    }
}

impl DeterministicState for ExchangeRuntime {
    fn state_hash(&self) -> [u8; 32] {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.risk_engine.state_hash());
        for (_, market) in self.markets.iter() {
            bytes.extend_from_slice(&market.book.state_hash());
        }
        bytes.extend_from_slice(&self.position_engine.state_hash());
        bytes.extend_from_slice(&self.ledger.state_hash());
        if let Some(seq) = self.last_applied_sequence_id {
            bytes.extend_from_slice(&seq.to_le_bytes());
        }
        hash_bytes(&bytes)
    }
}
