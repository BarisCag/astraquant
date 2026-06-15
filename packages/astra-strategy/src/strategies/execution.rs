use crate::agent::{AgentContext, StrategyAgent};
use crate::types::{MarketEvent, StrategyAction};
use astra_core::types::{Price, Quantity};
use astra_lob::types::OrderSide;

pub struct TwapExecutor {
    pub symbol: String,
    pub side: OrderSide,
    pub target_quantity: u64,
    pub completed_quantity: u64,
    pub slice_quantity: u64,
    pub interval_sequences: u64,
    pub last_execution_sequence: u64,
    pub next_order_id: u64,
}

impl TwapExecutor {
    pub fn new(
        symbol: String,
        side: OrderSide,
        target_quantity: u64,
        slice_quantity: u64,
        interval_sequences: u64,
    ) -> Self {
        Self {
            symbol,
            side,
            target_quantity,
            completed_quantity: 0,
            slice_quantity,
            interval_sequences,
            last_execution_sequence: 0,
            next_order_id: 1,
        }
    }
}

impl StrategyAgent for TwapExecutor {
    fn on_market_event(&mut self, event: &MarketEvent, ctx: &AgentContext) -> Vec<StrategyAction> {
        if self.completed_quantity >= self.target_quantity {
            return Vec::new();
        }

        let mut actions = Vec::new();

        let should_execute =
            ctx.engine_sequence_id >= self.last_execution_sequence + self.interval_sequences;

        if should_execute {
            if let MarketEvent::BookUpdate {
                best_bid, best_ask, ..
            } = event
            {
                // Cross the spread to execute immediately
                let price = match self.side {
                    OrderSide::Bid => best_ask.map(|p| p.0).unwrap_or(i64::MAX), // Pay whatever ask is
                    OrderSide::Ask => best_bid.map(|p| p.0).unwrap_or(0), // Sell to whatever bid is
                };

                let remaining = self.target_quantity - self.completed_quantity;
                let exec_qty = std::cmp::min(self.slice_quantity, remaining);

                actions.push(StrategyAction::SubmitOrder {
                    symbol: self.symbol.clone(),
                    side: self.side,
                    price: Price::new(price),
                    quantity: Quantity::new(exec_qty),
                });

                self.last_execution_sequence = ctx.engine_sequence_id;
                self.next_order_id += 1;
            }
        }

        actions
    }

    fn on_fill(
        &mut self,
        _order_id: u64,
        quantity: Quantity,
        _price: i64,
        _ctx: &AgentContext,
    ) -> Vec<StrategyAction> {
        self.completed_quantity += quantity.0;
        Vec::new()
    }

    fn on_risk_violation(&mut self, _reason: &str, _ctx: &AgentContext) -> Vec<StrategyAction> {
        Vec::new()
    }

    fn state_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.completed_quantity.to_le_bytes());
        hasher.update(&self.last_execution_sequence.to_le_bytes());
        *hasher.finalize().as_bytes()
    }
}

pub struct VwapExecutor {
    // Similar to TWAP but adjusts slice_quantity based on market volume.
    // For now, it will be a skeleton or simpler implementation.
    pub symbol: String,
}

// TODO: full implementation for VWAP
