use std::collections::HashMap;
use astra_core::events::{AstraEvent, EventType, PayloadMetadata, PayloadEncoding};
use crate::events::{PaperEvent, PortfolioSnapshot, RiskAlert};
use crate::execution::PaperExecutionEngine;
use crate::portfolio::PortfolioTracker;
use crate::prng::DeterministicPrng;
use crate::risk::RiskEngine;
use crate::strategy::{Strategy, StrategyContext};
use crate::types::{MarketSnapshot, Side};

pub struct PaperEngine {
    pub portfolio: PortfolioTracker,
    pub execution: PaperExecutionEngine,
    pub risk: RiskEngine,
    pub strategy: Box<dyn Strategy>,
    pub current_prices: HashMap<String, u64>,
    pub global_nonce: u64,
    pub last_snapshot_time: u64,
    pub last_nav: i64,
}

impl PaperEngine {
    pub fn new(
        portfolio: PortfolioTracker,
        execution: PaperExecutionEngine,
        risk: RiskEngine,
        strategy: Box<dyn Strategy>,
    ) -> Self {
        let initial_nav = portfolio.cash_balance;
        Self {
            portfolio,
            execution,
            risk,
            strategy,
            current_prices: HashMap::new(),
            global_nonce: 0,
            last_snapshot_time: 0,
            last_nav: initial_nav,
        }
    }

    pub fn process_event(&mut self, in_event: &AstraEvent, prev_journal_hash: &[u8; 32]) -> Vec<AstraEvent> {
        self.global_nonce += 1;
        let mut out_events = Vec::new();

        // Decode market data if any
        let snapshot = if in_event.event_type == EventType::MarketTick {
            // Assuming payload contains something we can parse into MarketSnapshot.
            // For now, let's mock it using normalizer or just assume it is MarketSnapshot struct directly?
            // Wait, AstraEvent in Phase 1 comes from TradeNormalizer. We will extract price directly.
            // Let's assume we can deserialize it as TradePayload or just MarketSnapshot.
            // Actually, Phase 1 creates MarketTick. 
            if let Ok(tick) = astra_core::serialization::deserialize_canonical::<astra_core::marketdata::MarketTick>(&in_event.payload) {
                let mid = (tick.bid_price.0 as u64 + tick.ask_price.0 as u64) / 2;
                self.current_prices.insert(tick.symbol.clone(), mid);
                Some(MarketSnapshot {
                    symbol: tick.symbol,
                    last_price: mid,
                    timestamp_ns: in_event.timestamp_ns,
                })
            } else {
                None
            }
        } else {
            None
        };

        // 1. Snapshot Portfolio if needed
        let current_nav = self.portfolio.total_equity(&self.current_prices);
        let time_since_last = in_event.timestamp_ns.saturating_sub(self.last_snapshot_time);
        
        let mut nav_diff = (current_nav - self.last_nav).abs() as f64 / self.last_nav.max(1) as f64;
        if self.last_nav == 0 { nav_diff = 0.0; }

        if time_since_last >= 1_000_000_000 || nav_diff > 0.01 {
            let snap = PortfolioSnapshot {
                timestamp_ns: in_event.timestamp_ns,
                cash_balance: self.portfolio.cash_balance,
                realized_pnl: self.portfolio.realized_pnl,
                unrealized_pnl: self.portfolio.unrealized_pnl(&self.current_prices),
                total_nav: current_nav,
            };
            out_events.push(self.wrap_event(
                in_event.timestamp_ns,
                EventType::StateSnapshot,
                PaperEvent::PortfolioSnapshot(snap),
            ));
            self.last_snapshot_time = in_event.timestamp_ns;
            self.last_nav = current_nav;
        }

        // 2. PRNG setup
        let mut prng = DeterministicPrng::new(
            prev_journal_hash,
            in_event.timestamp_ns,
            in_event.event_type as u8,
            self.global_nonce,
        );

        let mut ctx = StrategyContext {
            current_time_ns: in_event.timestamp_ns,
        };

        // 3. Market Data
        if let Some(ref snap) = snapshot {
            let orders = self.strategy.on_market_data(snap, &ctx);
            if !orders.is_empty() {
                out_events.push(self.wrap_event(
                    in_event.timestamp_ns,
                    EventType::OrderSubmitted,
                    PaperEvent::StrategySignal(orders.clone()),
                ));

                for order in orders {
                    let current_price = *self.current_prices.get(&order.symbol).unwrap_or(&0);
                    if let Err(reason) = self.risk.filter_order(&order, in_event.timestamp_ns, current_price, &self.portfolio, current_nav) {
                        out_events.push(self.wrap_event(
                            in_event.timestamp_ns,
                            EventType::RiskLimitBreached,
                            PaperEvent::RiskBreach(RiskAlert {
                                timestamp_ns: in_event.timestamp_ns,
                                reason: reason.clone(),
                                halted: true,
                            }),
                        ));
                        self.flatten_all(&mut out_events, in_event.timestamp_ns, snap.last_price, &snap.symbol);
                        return out_events;
                    }

                    if let Some(fill) = self.execution.try_fill(&order, snap, &mut prng, self.global_nonce) {
                        self.portfolio.update_from_fill(&fill);
                        self.strategy.on_fill(&fill, &mut ctx);
                        out_events.push(self.wrap_event(
                            in_event.timestamp_ns,
                            EventType::OrderFilled,
                            PaperEvent::Fill(fill),
                        ));
                    }
                }
            }
        }

        // 4. Clock
        let clock_orders = self.strategy.on_clock(in_event.timestamp_ns, &mut ctx);
        if !clock_orders.is_empty() {
            out_events.push(self.wrap_event(
                in_event.timestamp_ns,
                EventType::OrderSubmitted,
                PaperEvent::StrategySignal(clock_orders.clone()),
            ));

            for order in clock_orders {
                let current_price = *self.current_prices.get(&order.symbol).unwrap_or(&0);
                if let Err(reason) = self.risk.filter_order(&order, in_event.timestamp_ns, current_price, &self.portfolio, current_nav) {
                    out_events.push(self.wrap_event(
                        in_event.timestamp_ns,
                        EventType::RiskLimitBreached,
                        PaperEvent::RiskBreach(RiskAlert {
                            timestamp_ns: in_event.timestamp_ns,
                            reason: reason.clone(),
                            halted: true,
                        }),
                    ));
                    // Fake symbol here, real flatten should walk all positions
                    self.flatten_all(&mut out_events, in_event.timestamp_ns, current_price, &order.symbol);
                    return out_events;
                }

                if let Some(snap) = snapshot.as_ref() {
                    if let Some(fill) = self.execution.try_fill(&order, snap, &mut prng, self.global_nonce) {
                        self.portfolio.update_from_fill(&fill);
                        self.strategy.on_fill(&fill, &mut ctx);
                        out_events.push(self.wrap_event(
                            in_event.timestamp_ns,
                            EventType::OrderFilled,
                            PaperEvent::Fill(fill),
                        ));
                    }
                }
            }
        }

        out_events
    }

    fn flatten_all(&mut self, out_events: &mut Vec<AstraEvent>, timestamp_ns: u64, price: u64, _fallback_symbol: &str) {
        // Issue flatten orders for all open positions
        let mut flatten_fills = Vec::new();
        for (sym, pos) in &self.portfolio.positions {
            if pos.quantity == 0 { continue; }
            let side = if pos.quantity > 0 { Side::Sell } else { Side::Buy };
            let qty = pos.quantity.unsigned_abs();

            let current_price = *self.current_prices.get(sym).unwrap_or(&price);

            let fill = crate::events::FillEvent {
                symbol: sym.clone(),
                side,
                fill_price: current_price,
                fill_quantity: qty,
                timestamp_ns,
                model_used: "Flatten".to_string(),
                prng_nonce_at_fill: self.global_nonce,
            };
            flatten_fills.push(fill);
        }

        let mut ctx = StrategyContext { current_time_ns: timestamp_ns };

        for fill in flatten_fills {
            self.portfolio.update_from_fill(&fill);
            self.strategy.on_fill(&fill, &mut ctx);
            out_events.push(self.wrap_event(
                timestamp_ns,
                EventType::OrderFilled,
                PaperEvent::Fill(fill),
            ));
        }

        // Disable strategy
        let dummy_bytes = Vec::new(); // Ideally we tell the strategy to disable itself or set a flag here
        self.strategy.deserialize_state(&dummy_bytes);
    }

    fn wrap_event(&self, timestamp_ns: u64, event_type: EventType, paper: PaperEvent) -> AstraEvent {
        let payload = astra_core::serialization::serialize_canonical(&paper).unwrap();
        AstraEvent::new(
            timestamp_ns,
            0, // Assigned by gateway
            event_type,
            payload,
            PayloadMetadata::new(PayloadEncoding::Bincode, 1),
        )
    }
}
