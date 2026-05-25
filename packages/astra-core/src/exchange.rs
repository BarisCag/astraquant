//! Exchange runtime: matching engine, portfolio, ledger, and risk limits.

use crate::events::{AstraEvent, EventType};
use crate::hashing::{hash_bytes, DeterministicState};
use crate::ledger::TradeLedger;
use crate::matching::MatchingEngine;
use crate::orderbook::{
    LimitOrderCancelledPayload, LimitOrderMatchedPayload, LimitOrderPlacedPayload, OrderSide,
};
use crate::portfolio::Portfolio;
use crate::replay::EventReducer;
use crate::risk::RiskLimits;
use crate::serialization::deserialize_canonical;
use crate::trades::{Trade, TradeId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExchangeRuntime {
    pub risk_limits: RiskLimits,
    pub markets: BTreeMap<String, MatchingEngine>,
    pub portfolio: Portfolio,
    pub ledger: TradeLedger,
    pub last_applied_sequence_id: Option<u64>,
}

impl ExchangeRuntime {
    pub fn new(risk_limits: RiskLimits) -> Self {
        Self {
            risk_limits,
            markets: BTreeMap::new(),
            portfolio: Portfolio::new(),
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
            price: match_payload.match_price,
            quantity: match_payload.matched_quantity,
            timestamp_ns,
        };
        self.portfolio.apply_trade_settled(
            &trade.symbol,
            trade.taker_side,
            trade.price,
            trade.quantity,
        );
        self.ledger.trades.insert(trade.trade_id.0, trade);
    }

    fn apply_limit_order_placed(
        &mut self,
        event: &AstraEvent,
        payload: &LimitOrderPlacedPayload,
    ) -> Result<(), String> {
        self.risk_limits.validate_order(
            &self.portfolio,
            &payload.symbol,
            payload.quantity,
            payload.price,
        )?;

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
                    self.portfolio.apply_trade_settled(
                        &trade.symbol,
                        trade.taker_side,
                        trade.price,
                        trade.quantity,
                    );
                    self.ledger.trades.insert(trade.trade_id.0, trade);
                }
            }
            EventType::RiskLimitBreached => {
                self.risk_limits.apply(event)?;
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
        bytes.extend_from_slice(&self.risk_limits.state_hash());
        for (_, market) in self.markets.iter() {
            bytes.extend_from_slice(&market.book.state_hash());
        }
        bytes.extend_from_slice(&self.portfolio.state_hash());
        bytes.extend_from_slice(&self.ledger.state_hash());
        if let Some(seq) = self.last_applied_sequence_id {
            bytes.extend_from_slice(&seq.to_le_bytes());
        }
        hash_bytes(&bytes)
    }
}
