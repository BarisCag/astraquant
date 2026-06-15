use crate::agent::{AgentContext, StrategyAgent};
use crate::types::{MarketEvent, StrategyAction};
use astra_core::types::{Price, Quantity};
use astra_lob::types::OrderSide;
use std::collections::VecDeque;

pub struct CancellationAgent {
    pub symbol: String,
    pub max_active_orders: usize,
    pub active_orders: VecDeque<u64>,
    pub next_order_id: u64,
}

impl CancellationAgent {
    pub fn new(symbol: String, max_active_orders: usize) -> Self {
        Self {
            symbol,
            max_active_orders,
            active_orders: VecDeque::new(),
            next_order_id: 1,
        }
    }
}

impl StrategyAgent for CancellationAgent {
    fn on_market_event(&mut self, event: &MarketEvent, _ctx: &AgentContext) -> Vec<StrategyAction> {
        let mut actions = Vec::new();

        if let MarketEvent::BookUpdate {
            best_bid, best_ask, ..
        } = event
        {
            // Cancel oldest if we have too many
            while self.active_orders.len() >= self.max_active_orders {
                if let Some(id) = self.active_orders.pop_front() {
                    actions.push(StrategyAction::CancelOrder {
                        symbol: self.symbol.clone(),
                        order_id: id,
                    });
                }
            }

            // Submit new order deep in the book to not match immediately
            let target_price = if let Some(bid) = best_bid {
                bid.0 - 1000 // deep bid
            } else if let Some(ask) = best_ask {
                ask.0 - 1000
            } else {
                10000
            };

            let id = self.next_order_id;
            self.next_order_id += 1;
            self.active_orders.push_back(id);

            actions.push(StrategyAction::SubmitOrder {
                symbol: self.symbol.clone(),
                side: OrderSide::Bid,
                price: Price::new(target_price),
                quantity: Quantity::new(1),
            });
        }

        actions
    }

    fn on_fill(
        &mut self,
        _order_id: u64,
        _quantity: Quantity,
        _price: i64,
        _ctx: &AgentContext,
    ) -> Vec<StrategyAction> {
        Vec::new()
    }

    fn on_risk_violation(&mut self, _reason: &str, _ctx: &AgentContext) -> Vec<StrategyAction> {
        Vec::new()
    }

    fn state_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.next_order_id.to_le_bytes());
        *hasher.finalize().as_bytes()
    }
}

pub struct SweeperAgent {
    pub symbol: String,
    pub sweep_interval_sequences: u64,
    pub last_sweep_sequence: u64,
}

impl SweeperAgent {
    pub fn new(symbol: String, sweep_interval_sequences: u64) -> Self {
        Self {
            symbol,
            sweep_interval_sequences,
            last_sweep_sequence: 0,
        }
    }
}

impl StrategyAgent for SweeperAgent {
    fn on_market_event(&mut self, _event: &MarketEvent, ctx: &AgentContext) -> Vec<StrategyAction> {
        let mut actions = Vec::new();
        if ctx.engine_sequence_id >= self.last_sweep_sequence + self.sweep_interval_sequences {
            actions.push(StrategyAction::SubmitOrder {
                symbol: self.symbol.clone(),
                side: OrderSide::Bid,
                price: Price::new(i64::MAX),  // market order sweep
                quantity: Quantity::new(100), // large quantity
            });
            self.last_sweep_sequence = ctx.engine_sequence_id;
        }
        actions
    }

    fn on_fill(
        &mut self,
        _order_id: u64,
        _quantity: Quantity,
        _price: i64,
        _ctx: &AgentContext,
    ) -> Vec<StrategyAction> {
        Vec::new()
    }

    fn on_risk_violation(&mut self, _reason: &str, _ctx: &AgentContext) -> Vec<StrategyAction> {
        Vec::new()
    }

    fn state_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.last_sweep_sequence.to_le_bytes());
        *hasher.finalize().as_bytes()
    }
}
