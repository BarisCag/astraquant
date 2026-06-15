use crate::observability::ExchangeDiagnostics;
use crate::state::ExchangeStateHash;
use astra_core::events::{AstraEvent, EventType};
use astra_core::orderbook::{LimitOrderPlacedPayload, OrderSide};
use astra_core::serialization::deserialize_canonical;
use astra_lob::book::LimitOrderBook;
use astra_lob::types::OrderEvent;
use astra_portfolio::engine::PositionEngine;
use astra_risk::engine::RiskEngine;
use astra_router::router::SmartOrderRouter;
use astra_router::venue::{VenueId, VenueState, VenueFeeModel, VenueLatencyProfile};
use bincode::Options;
use std::collections::BTreeMap;
use astra_clearing::settlement::SettlementEngine;
use astra_clearing::margin::MarginEngine;
use astra_clearing::funding::FundingLedger;
use astra_strategy::runtime::StrategyRuntime;
use astra_strategy::types::{StrategyAction, MarketEvent};
use astra_core::events::{PayloadEncoding, PayloadMetadata};
use astra_core::orderbook::{LimitOrderCancelledPayload, OrderSide as CoreOrderSide};
use astra_core::serialization::serialize_canonical;
use astra_ops::control::{OperationalAction, OperationalCommand};

pub struct ExchangeRuntime {
    pub risk_engine: RiskEngine,
    pub router: SmartOrderRouter,
    pub position_engine: PositionEngine,
    pub diagnostics: ExchangeDiagnostics,
    pub sequence_clock: u64,
    pub strategy_runtime: StrategyRuntime,
    pub settlement_engine: SettlementEngine,
    pub margin_engine: MarginEngine,
    pub funding_ledger: FundingLedger,
    pub policy_engine: astra_policy::engine::PolicyEngine,
    pub ecology_orchestrator: astra_agents::ecology::EcologyOrchestrator,
}

impl ExchangeRuntime {
    pub fn new(risk_engine: RiskEngine) -> Self {
        let mut router = SmartOrderRouter::new();
        // Initialize with one default venue for now
        let venue_state = VenueState::new(
            VenueId(1),
            VenueFeeModel { maker_fee_bps: 0, taker_fee_bps: 0 },
            VenueLatencyProfile { ingress_delay_sequences: 1 },
        );
        router.add_venue(venue_state);

        Self {
            risk_engine,
            router,
            position_engine: PositionEngine::new(),
            diagnostics: ExchangeDiagnostics::default(),
            sequence_clock: 0,
            strategy_runtime: StrategyRuntime::new(),
            settlement_engine: SettlementEngine::new(2),
            margin_engine: MarginEngine::new(),
            funding_ledger: FundingLedger::new(),
            policy_engine: astra_policy::engine::PolicyEngine::new(),
            ecology_orchestrator: astra_agents::ecology::EcologyOrchestrator::new(
                astra_agents::ecology::AgentEcology {
                    agents: std::collections::BTreeMap::new(),
                }
            ),
        }
    }

    pub fn apply_event(&mut self, event: &AstraEvent) -> Result<Vec<AstraEvent>, String> {
        self.diagnostics.total_events_processed += 1;
        self.sequence_clock = event.sequence_id;
        self.router.sequence_clock = event.sequence_id;
        self.risk_engine.increment_sequence();

        // 1. Risk validation and Router queuing for top-level events
        match event.event_type {
            EventType::LimitOrderPlaced => {
                let payload: LimitOrderPlacedPayload =
                    deserialize_canonical(&event.payload).map_err(|e| e.to_string())?;

                let notional = payload.price.0.saturating_mul(payload.quantity.0 as i64);

                // Risk Validation
                if let Err(e) =
                    self.risk_engine
                        .validate_order(payload.trader_id, payload.quantity.0, notional)
                {
                    self.diagnostics.total_rejected_orders += 1;
                    
                    let strategy_actions = self.strategy_runtime.dispatch_risk_violation(payload.trader_id, &format!("{:?}", e));
                    return Ok(self.convert_strategy_actions_to_events(strategy_actions, event.sequence_id, event.timestamp_ns));
                }
                self.diagnostics.total_accepted_orders += 1;
                self.router.route_order(event.clone(), None);
            }
            EventType::LimitOrderCancelled => {
                self.router.route_order(event.clone(), None);
            }
            EventType::VenueStatusChanged => {
                if let Ok(failure_event) = deserialize_canonical::<astra_router::failure::VenueFailureEvent>(&event.payload) {
                    match failure_event {
                        astra_router::failure::VenueFailureEvent::VenueOffline { venue_id, .. } => {
                            if let Some(v) = self.router.venues.get_mut(&venue_id) {
                                v.status = astra_router::venue::VenueStatus::Offline;
                            }
                        }
                        astra_router::failure::VenueFailureEvent::VenuePaused { venue_id, .. } => {
                            if let Some(v) = self.router.venues.get_mut(&venue_id) {
                                v.status = astra_router::venue::VenueStatus::Paused;
                            }
                        }
                        astra_router::failure::VenueFailureEvent::VenueRecovered { venue_id, .. } => {
                            if let Some(v) = self.router.venues.get_mut(&venue_id) {
                                v.status = astra_router::venue::VenueStatus::Active;
                            }
                        }
                        _ => {}
                    }
                }
            }
            EventType::MarketStressInjected => {
                if let Ok(failure_event) = deserialize_canonical::<astra_router::failure::VenueFailureEvent>(&event.payload) {
                    match failure_event {
                        astra_router::failure::VenueFailureEvent::LiquidityCollapse { venue_id, symbol, fraction_to_remove: _, .. } => {
                            if let Some(venue) = self.router.venues.get_mut(&venue_id) {
                                if let Some(book) = venue.books.get_mut(&symbol) {
                                    let events = astra_router::stress::LiquidityCollapseModel::apply(book, 10); // arbitrary max
                                    self.diagnostics.lob_diagnostics.ingest_events(&events);
                                    self.diagnostics.lob_diagnostics.update_depth_metrics(book);
                                }
                            }
                        }
                        astra_router::failure::VenueFailureEvent::SpreadExpansion { venue_id, symbol, tick_expansion, .. } => {
                            if let Some(venue) = self.router.venues.get_mut(&venue_id) {
                                if let Some(book) = venue.books.get_mut(&symbol) {
                                    let events = astra_router::stress::SpreadExpansionModel::apply(book, tick_expansion as i64);
                                    self.diagnostics.lob_diagnostics.ingest_events(&events);
                                    self.diagnostics.lob_diagnostics.update_depth_metrics(book);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            EventType::MarketTick => {
                // Determine endogenous policy actions (stubbed for now)
                let _actions = self.policy_engine.evaluate_sequence(
                    event.sequence_id,
                    &astra_policy::systemic::SystemicPropagationMetrics::new()
                );
            }
            EventType::OperatorAction => {
                let bytes = event.payload.as_slice();
                if let Ok(action) = bincode::options().with_little_endian().with_fixint_encoding().deserialize::<astra_ops::control::OperationalAction>(bytes) {
                    match action.command {
                        astra_ops::control::OperationalCommand::PauseVenue { venue_id } => {
                            if let Some(venue) = self.router.venues.get_mut(&astra_router::venue::VenueId(venue_id)) {
                                venue.status = astra_router::venue::VenueStatus::Offline;
                            }
                        }
                        astra_ops::control::OperationalCommand::ResumeVenue { venue_id } => {
                            if let Some(venue) = self.router.venues.get_mut(&astra_router::venue::VenueId(venue_id)) {
                                venue.status = astra_router::venue::VenueStatus::Active;
                            }
                        }
                        astra_ops::control::OperationalCommand::InjectRecoveryLiquidity { symbol, size, price } => {
                            let _ = symbol;
                            let _ = size;
                            let _ = price;
                        }
                        _ => {}
                    }
                }
            }
            EventType::PolicyAction | EventType::RegulatoryIntervention | EventType::LiquidityFacilityActivated | EventType::CircuitBreakerTriggered | EventType::SettlementHolidayActivated => {
                // Explicitly consume policy events preserving sequence progression and deterministic state hash.
                // In a full implementation, we'd apply collateral easing to the risk engine or halt trading venues.
            }
            EventType::AuditCheckpoint | EventType::InvariantViolationDetected | EventType::ReplayVerificationCompleted => {
                // Audit events are consumed for sequence progression only.
                // They do not mutate exchange state — they are verification artifacts.
            }
            EventType::AgentIntent | EventType::BehaviorTransition | EventType::SystemicCascadeTriggered | EventType::AgentLiquidityWithdrawal | EventType::AgentMarginDefense => {
                // Phase 14A: Agent events are ingested canonically.
                // Intent execution (if any) would map onto standard matching engine operations.
                // Ecology events are consumed sequentially, preserving replay lineage.
            }
            _ => {}
        }

        // 2. Advance clock and process arrived scheduled orders at venues
        let scheduled_orders = self.router.advance_clock(event.sequence_id);
        let mut strategy_actions: Vec<(u64, StrategyAction)> = Vec::new();

        for scheduled in scheduled_orders {
            if let Some(venue) = self.router.venues.get_mut(&scheduled.venue_id) {
                let venue_event = &scheduled.order_payload;

                if venue.status == astra_router::venue::VenueStatus::Offline {
                    if venue_event.event_type == EventType::LimitOrderPlaced {
                        if let Ok(payload) = deserialize_canonical::<LimitOrderPlacedPayload>(&venue_event.payload) {
                            let reject_event = OrderEvent::DestinationUnavailable {
                                order_id: payload.order_id,
                                venue_id: venue.venue_id.0,
                                reason: "Venue Offline".to_string(),
                            };
                            self.diagnostics.lob_diagnostics.ingest_events(&[reject_event]);
                        }
                    }
                    continue;
                }

                match venue_event.event_type {
                    EventType::LimitOrderPlaced => {
                        let payload: LimitOrderPlacedPayload =
                            deserialize_canonical(&venue_event.payload).map_err(|e| e.to_string())?;

                        let book = venue
                            .books
                            .entry(payload.symbol.clone())
                            .or_insert_with(|| LimitOrderBook::new(payload.symbol.clone()));

                        let side = match payload.side {
                            OrderSide::Bid => astra_lob::types::OrderSide::Bid,
                            OrderSide::Ask => astra_lob::types::OrderSide::Ask,
                        };

                        let order = astra_lob::types::Order {
                            order_id: payload.order_id,
                            symbol: payload.symbol.clone(),
                            trader_id: payload.trader_id,
                            side,
                            order_type: astra_lob::types::OrderType::Limit,
                            price: payload.price,
                            quantity: payload.quantity,
                            remaining_quantity: payload.quantity,
                            timestamp_ns: venue_event.timestamp_ns,
                            queue_position: Default::default(),
                        };

                        let events = book.submit(order);
                        self.diagnostics.lob_diagnostics.ingest_events(&events);
                        self.diagnostics.lob_diagnostics.update_depth_metrics(book);

                        let snapshot = book.snapshot();
                        let actions = self.strategy_runtime.dispatch_market_event(&MarketEvent::BookUpdate {
                            engine_sequence_id: event.sequence_id,
                            symbol: payload.symbol.clone(),
                            best_bid: snapshot.best_bid,
                            best_ask: snapshot.best_ask,
                            bid_depth: snapshot.total_bid_liquidity,
                            ask_depth: snapshot.total_ask_liquidity,
                        });
                        strategy_actions.extend(actions);

                        for lob_event in events {
                            if let OrderEvent::TradeExecuted(execution) = lob_event {
                                let is_buy = execution.liquidity_side
                                    == astra_lob::types::LiquiditySide::Taker
                                    && payload.side == OrderSide::Bid
                                    || execution.liquidity_side == astra_lob::types::LiquiditySide::Maker
                                        && payload.side == OrderSide::Ask;

                                self.position_engine.apply_fill(
                                    execution.trader_id,
                                    &payload.symbol,
                                    is_buy,
                                    execution.matched_quantity.0,
                                    execution.match_price.0,
                                );
                                self.position_engine
                                    .update_mark_price(&payload.symbol, execution.match_price.0);
                                
                                if let Some(ctx) = self.strategy_runtime.contexts.get_mut(&execution.trader_id) {
                                    if let Some(pos) = self.position_engine.positions.get(&execution.trader_id).and_then(|m| m.get(&payload.symbol)) {
                                        ctx.inventory = pos.net_quantity;
                                        ctx.realized_pnl = pos.realized_pnl;
                                        ctx.unrealized_pnl = pos.unrealized_pnl;
                                    }
                                }

                                let acts = self.strategy_runtime.dispatch_fill(execution.trader_id, execution.resting_order_id, execution.matched_quantity, execution.match_price.0);
                                strategy_actions.extend(acts);
                                let acts_aggr = self.strategy_runtime.dispatch_fill(execution.trader_id, execution.aggressive_order_id, execution.matched_quantity, execution.match_price.0);
                                strategy_actions.extend(acts_aggr);

                                let trade_acts = self.strategy_runtime.dispatch_market_event(&MarketEvent::TradeExecution {
                                    engine_sequence_id: event.sequence_id,
                                    symbol: payload.symbol.clone(),
                                    price: execution.match_price,
                                    quantity: execution.matched_quantity,
                                    is_buyer_maker: execution.liquidity_side == astra_lob::types::LiquiditySide::Maker,
                                });
                                strategy_actions.extend(trade_acts);

                                self.settlement_engine.queue_trade(
                                    execution.trader_id,
                                    payload.symbol.clone(),
                                    execution.matched_quantity.0,
                                    execution.match_price.0 as u64,
                                    is_buy,
                                    event.sequence_id
                                );
                            }
                        }
                    }
                    EventType::LimitOrderCancelled => {
                        let payload: LimitOrderCancelledPayload =
                            deserialize_canonical(&venue_event.payload).map_err(|e| e.to_string())?;
                        
                        if let Some(book) = venue.books.get_mut(&payload.symbol) {
                            let events = book.cancel(payload.order_id);
                            self.diagnostics.lob_diagnostics.ingest_events(&events);
                            self.diagnostics.lob_diagnostics.update_depth_metrics(book);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        let mature_settlements = self.settlement_engine.mature_obligations(event.sequence_id);
        for bucket in mature_settlements {
            self.funding_ledger.apply_cash_movement(bucket.trader_id, bucket.net_cash_movement);
            // Also need to update collateral in margin_engine
            let balance = self.funding_ledger.get_balance(bucket.trader_id);
            // For now, assume utilized margin is derived from positions.
            // A simplified position notional approximation:
            let mut utilized_margin = 0;
            if let Some(positions) = self.position_engine.positions.get(&bucket.trader_id) {
                for pos in positions.values() {
                    utilized_margin += (pos.net_quantity.abs() as u64) * pos.last_mark_price as u64;
                }
            }
            self.margin_engine.update_collateral(bucket.trader_id, balance, utilized_margin);
        }

        let liquidations = self.margin_engine.check_margin_health(event.sequence_id);
        for liq in liquidations {
            // Generate a forced liquidation market order StrategyAction
            strategy_actions.push((liq.trader_id, StrategyAction::SubmitOrder {
                symbol: liq.symbol.clone(),
                side: if liq.is_buy { astra_lob::types::OrderSide::Bid } else { astra_lob::types::OrderSide::Ask },
                price: astra_core::types::Price(if liq.is_buy { i64::MAX } else { 0 }), // Market order simulation
                quantity: astra_core::types::Quantity(liq.quantity),
            }));
            
            // Generate a LiquidationExecuted AstraEvent?
            // Will let the strategy_actions convert into LimitOrderPlaced.
        }
        
        if !strategy_actions.is_empty() {
            return Ok(self.convert_strategy_actions_to_events(strategy_actions, event.sequence_id, event.timestamp_ns));
        }

        Ok(Vec::new())
    }

    fn convert_strategy_actions_to_events(&self, actions: Vec<(u64, StrategyAction)>, _parent_seq: u64, timestamp_ns: u64) -> Vec<AstraEvent> {
        let mut events = Vec::new();
        // Since we are creating child events, we can use an offset from parent_seq, but to maintain strictly monotonic 
        // global sequence ids, the outer caller should probably manage it. Wait, the instructions say "convert StrategyActions into deterministic AstraEvents".
        // We will just generate AstraEvents with seq = 0 and let the Inspector/benchmark override it when inserting into journal!
        
        for (trader_id, action) in actions {
            match action {
                StrategyAction::SubmitOrder { symbol, side, price, quantity } => {
                    let payload = LimitOrderPlacedPayload {
                        order_id: 0, // Should be generated by caller
                        trader_id,
                        symbol,
                        side: match side {
                            astra_lob::types::OrderSide::Bid => CoreOrderSide::Bid,
                            astra_lob::types::OrderSide::Ask => CoreOrderSide::Ask,
                        },
                        price,
                        quantity,
                    };
                    events.push(AstraEvent {
                        sequence_id: 0,
                        event_type: EventType::LimitOrderPlaced,
                        timestamp_ns,
                        payload: serialize_canonical(&payload).unwrap(),
                        payload_metadata: PayloadMetadata::new(PayloadEncoding::Bincode, 1),
                    });
                }
                StrategyAction::CancelOrder { symbol, order_id } => {
                    let payload = LimitOrderCancelledPayload {
                        symbol,
                        order_id,
                    };
                    events.push(AstraEvent {
                        sequence_id: 0,
                        event_type: EventType::LimitOrderCancelled,
                        timestamp_ns,
                        payload: serialize_canonical(&payload).unwrap(),
                        payload_metadata: PayloadMetadata::new(PayloadEncoding::Bincode, 1),
                    });
                }
                _ => {}
            }
        }
        events
    }

    pub fn generate_global_hash(&self) -> ExchangeStateHash {
        let risk_engine_hash = self.risk_engine.state_hash();
        let portfolio_engine_hash = self.position_engine.state_hash();

        let mut lob_bytes = Vec::new();
        // Since we now have multiple venues, we should hash them hierarchically
        for venue in self.router.venues.values() {
            let mut venue_lob_bytes = Vec::new();
            for book in venue.books.values() {
                let book_bytes = bincode::options()
                    .with_little_endian()
                    .with_fixint_encoding()
                    .serialize(book)
                    .expect("Book serialization failed");
                let mut h = blake3::Hasher::new();
                h.update(&book_bytes);
                venue_lob_bytes.extend_from_slice(h.finalize().as_bytes());
            }
            lob_bytes.extend_from_slice(&venue.state_hash());
            lob_bytes.extend_from_slice(&venue_lob_bytes);
        }

        let mut hasher = blake3::Hasher::new();
        hasher.update(&lob_bytes);
        let matching_engines_hash = *hasher.finalize().as_bytes();

        let diagnostics_hash = self.diagnostics.state_hash();

        let mut settlement_bytes = bincode::options().with_little_endian().with_fixint_encoding().serialize(&self.settlement_engine).unwrap();
        let mut margin_bytes = bincode::options().with_little_endian().with_fixint_encoding().serialize(&self.margin_engine).unwrap();
        let mut funding_bytes = bincode::options().with_little_endian().with_fixint_encoding().serialize(&self.funding_ledger).unwrap();

        let mut h1 = blake3::Hasher::new(); h1.update(&settlement_bytes);
        let mut h2 = blake3::Hasher::new(); h2.update(&margin_bytes);
        let mut h3 = blake3::Hasher::new(); h3.update(&funding_bytes);

        ExchangeStateHash {
            risk_engine_hash,
            portfolio_engine_hash,
            matching_engines_hash,
            diagnostics_hash,
            strategy_runtime_hash: self.strategy_runtime.state_hash(),
            settlement_engine_hash: *h1.finalize().as_bytes(),
            margin_engine_hash: *h2.finalize().as_bytes(),
            funding_ledger_hash: *h3.finalize().as_bytes(),
            sequence_clock: self.sequence_clock,
        }
    }

    /// Phase 14A: Deterministic Ecology Evaluation
    /// Evaluates the multi-agent ecology at the current sequence and emits intents
    /// to be appended to the NEXT sequence journal.
    pub fn evaluate_ecology(&mut self, current_sequence: u64) -> Vec<astra_core::events::AstraEvent> {
        let batch = self.ecology_orchestrator.evaluate_sequence(current_sequence);
        let mut events = Vec::new();
        for intent in batch.intents {
            events.push(astra_agents::emission::IntentEmitter::emit(&intent, current_sequence + 1));
        }
        events
    }
}
