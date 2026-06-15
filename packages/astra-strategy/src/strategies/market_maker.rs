use crate::agent::{AgentContext, StrategyAgent};
use crate::types::{MarketEvent, StrategyAction};
use astra_core::types::{Price, Quantity};
use astra_lob::types::OrderSide;

pub struct PassiveMarketMaker {
    pub symbol: String,
    pub half_spread_ticks: i64,
    pub order_qty: u64,
    pub max_inventory: i64,
    pub skew_ticks_per_lot: i64,
    pub bid_order_id: Option<u64>,
    pub ask_order_id: Option<u64>,
    pub next_order_id: u64,
    pub current_bid_price: Option<i64>,
    pub current_ask_price: Option<i64>,
}

impl PassiveMarketMaker {
    pub fn new(
        symbol: String,
        half_spread_ticks: i64,
        order_qty: u64,
        max_inventory: i64,
        skew_ticks_per_lot: i64,
    ) -> Self {
        Self {
            symbol,
            half_spread_ticks,
            order_qty,
            max_inventory,
            skew_ticks_per_lot,
            bid_order_id: None,
            ask_order_id: None,
            next_order_id: 1,
            current_bid_price: None,
            current_ask_price: None,
        }
    }
}

impl StrategyAgent for PassiveMarketMaker {
    fn on_market_event(&mut self, event: &MarketEvent, ctx: &AgentContext) -> Vec<StrategyAction> {
        let mut actions = Vec::new();

        if let MarketEvent::BookUpdate {
            best_bid, best_ask, ..
        } = event
        {
            if let (Some(bid), Some(ask)) = (best_bid, best_ask) {
                let scaled_midpoint = bid.0 + ask.0;
                let mid = scaled_midpoint / 2;

                let skew = ctx.inventory * self.skew_ticks_per_lot;

                let target_bid = mid - self.half_spread_ticks - skew;
                let target_ask = mid + self.half_spread_ticks - skew;

                let mut need_requote_bid = false;
                let mut need_requote_ask = false;

                if self.current_bid_price != Some(target_bid) {
                    need_requote_bid = true;
                }
                if self.current_ask_price != Some(target_ask) {
                    need_requote_ask = true;
                }

                if need_requote_bid {
                    if let Some(id) = self.bid_order_id {
                        actions.push(StrategyAction::CancelOrder {
                            symbol: self.symbol.clone(),
                            order_id: id,
                        });
                        self.bid_order_id = None;
                        self.current_bid_price = None;
                    }
                    if ctx.inventory < self.max_inventory {
                        let id = self.next_order_id;
                        self.next_order_id += 1;
                        self.bid_order_id = Some(id);
                        self.current_bid_price = Some(target_bid);
                        actions.push(StrategyAction::SubmitOrder {
                            symbol: self.symbol.clone(),
                            side: OrderSide::Bid,
                            price: Price::new(target_bid),
                            quantity: Quantity::new(self.order_qty),
                        });
                    }
                }

                if need_requote_ask {
                    if let Some(id) = self.ask_order_id {
                        actions.push(StrategyAction::CancelOrder {
                            symbol: self.symbol.clone(),
                            order_id: id,
                        });
                        self.ask_order_id = None;
                        self.current_ask_price = None;
                    }
                    if ctx.inventory > -self.max_inventory {
                        let id = self.next_order_id;
                        self.next_order_id += 1;
                        self.ask_order_id = Some(id);
                        self.current_ask_price = Some(target_ask);
                        actions.push(StrategyAction::SubmitOrder {
                            symbol: self.symbol.clone(),
                            side: OrderSide::Ask,
                            price: Price::new(target_ask),
                            quantity: Quantity::new(self.order_qty),
                        });
                    }
                }
            }
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
        // We will just requote on next book update
        Vec::new()
    }

    fn on_risk_violation(&mut self, _reason: &str, _ctx: &AgentContext) -> Vec<StrategyAction> {
        Vec::new()
    }

    fn state_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.next_order_id.to_le_bytes());
        if let Some(p) = self.current_bid_price {
            hasher.update(&p.to_le_bytes());
        }
        if let Some(p) = self.current_ask_price {
            hasher.update(&p.to_le_bytes());
        }
        *hasher.finalize().as_bytes()
    }
}
