use crate::events::FillEvent;
use crate::types::{MarketSnapshot, PaperOrder, Side, OrderType};

pub struct StrategyContext {
    pub current_time_ns: u64,
    // Add additional context if needed (e.g., current nav, balances)
}

pub trait Strategy: Send + Sync {
    fn on_market_data(&mut self, snapshot: &MarketSnapshot, ctx: &StrategyContext) -> Vec<PaperOrder>;
    fn on_fill(&mut self, fill: &FillEvent, ctx: &mut StrategyContext);
    fn on_clock(&mut self, now_ns: u64, ctx: &mut StrategyContext) -> Vec<PaperOrder>;
    fn serialize_state(&self) -> Vec<u8>;
    fn deserialize_state(&mut self, bytes: &[u8]);
}

pub struct TwapStrategy {
    pub symbol: String,
    pub side: Side,
    pub total_quantity: u64,
    pub quantity_per_slice: u64,
    pub interval_ns: u64,
    pub next_execution_time: u64,
    pub executed_quantity: u64,
    pub active: bool,
}

impl TwapStrategy {
    pub fn new(
        symbol: String,
        side: Side,
        total_quantity: u64,
        quantity_per_slice: u64,
        interval_ns: u64,
        start_time_ns: u64,
    ) -> Self {
        Self {
            symbol,
            side,
            total_quantity,
            quantity_per_slice,
            interval_ns,
            next_execution_time: start_time_ns,
            executed_quantity: 0,
            active: true,
        }
    }
}

impl Strategy for TwapStrategy {
    fn on_market_data(&mut self, _snapshot: &MarketSnapshot, _ctx: &StrategyContext) -> Vec<PaperOrder> {
        vec![]
    }

    fn on_fill(&mut self, fill: &FillEvent, _ctx: &mut StrategyContext) {
        if fill.symbol == self.symbol && fill.side == self.side {
            self.executed_quantity += fill.fill_quantity;
            if self.executed_quantity >= self.total_quantity {
                self.active = false;
            }
        }
    }

    fn on_clock(&mut self, now_ns: u64, _ctx: &mut StrategyContext) -> Vec<PaperOrder> {
        if !self.active || now_ns < self.next_execution_time {
            return vec![];
        }

        let remaining = self.total_quantity.saturating_sub(self.executed_quantity);
        if remaining == 0 {
            self.active = false;
            return vec![];
        }

        let slice_qty = std::cmp::min(self.quantity_per_slice, remaining);
        self.next_execution_time = now_ns + self.interval_ns; // ensure we step forward from now_ns if we fell behind

        vec![PaperOrder {
            symbol: self.symbol.clone(),
            side: self.side,
            order_type: OrderType::Market,
            quantity: slice_qty,
        }]
    }

    fn serialize_state(&self) -> Vec<u8> {
        // Serialize strategy state
        bincode::serialize(&self.executed_quantity).unwrap()
    }

    fn deserialize_state(&mut self, bytes: &[u8]) {
        self.executed_quantity = bincode::deserialize(bytes).unwrap_or(self.executed_quantity);
        if self.executed_quantity >= self.total_quantity {
            self.active = false;
        }
    }
}
