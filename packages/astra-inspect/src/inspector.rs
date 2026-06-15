use crate::analytics::{BenchmarkReport, OrderFlowAnalytics};
use crate::timeline::ReplayTimeline;
use astra_core::events::{AstraEvent, EventType};
use astra_core::journal::EventJournal;
use astra_core::orderbook::{LimitOrderPlacedPayload, OrderSide};
use astra_core::serialization::deserialize_canonical;
use astra_exchange::runtime::ExchangeRuntime;
use astra_lob::book::LimitOrderBook;
use astra_lob::types::OrderEvent;
use astra_stream::replay::collect_journal_files;
use crate::execution_quality::ExecutionQualityMetrics;
use crate::strategy_analytics::StrategyAnalyticsCollector;
use std::path::Path;

pub struct ReplayInspector {
    pub runtime: ExchangeRuntime,
    pub analytics: OrderFlowAnalytics,
    pub timeline: ReplayTimeline,
    pub execution_quality: ExecutionQualityMetrics,
    pub strategy_analytics: StrategyAnalyticsCollector,
}

impl ReplayInspector {
    pub fn new(runtime: ExchangeRuntime) -> Self {
        Self {
            runtime,
            analytics: OrderFlowAnalytics::new(),
            timeline: ReplayTimeline::new(),
            execution_quality: ExecutionQualityMetrics::new(),
            strategy_analytics: StrategyAnalyticsCollector::new(),
        }
    }

    pub fn inspect_directory(&mut self, journal_dir: &Path) -> Result<BenchmarkReport, String> {
        let start_time = std::time::Instant::now();
        let mut files = collect_journal_files(journal_dir).map_err(|e| e.to_string())?;
        files.sort();

        let mut total_bytes = 0;

        for file in files {
            total_bytes += std::fs::metadata(&file).map_err(|e| e.to_string())?.len();
            let iter = EventJournal::iter_path(&file).map_err(|e| e.to_string())?;

            for event_res in iter {
                let event = event_res.map_err(|e| e.to_string())?;
                self.process_event_for_inspection(&event)?;
                let mut children = self.runtime.apply_event(&event)?;
                while !children.is_empty() {
                    let mut next_gen = Vec::new();
                    for child in children {
                        self.process_event_for_inspection(&child)?;
                        let mut gc = self.runtime.apply_event(&child)?;
                        next_gen.append(&mut gc);
                    }
                    children = next_gen;
                }
            }
            // Capture metrics
            for (&trader_id, ctx) in &self.runtime.strategy_runtime.contexts {
                self.strategy_analytics.record_context(self.runtime.sequence_clock, trader_id, ctx);
            }
        }

        let elapsed_us = start_time.elapsed().as_micros() as u64;
        let events_per_sec = self
            .runtime
            .diagnostics
            .total_events_processed
            .saturating_mul(1_000_000)
            .checked_div(elapsed_us)
            .unwrap_or(0);
        let trades_per_sec = self
            .analytics
            .total_fills
            .saturating_mul(1_000_000)
            .checked_div(elapsed_us)
            .unwrap_or(0);

        // Compute PnL
        let mut realized_pnl = 0;
        let mut unrealized_pnl = 0;
        for exp in self
            .runtime
            .position_engine
            .positions
            .values()
            .flat_map(|m| m.values())
        {
            realized_pnl += exp.realized_pnl;
            unrealized_pnl += exp.unrealized_pnl;
        }

        let mut venue_analytics = std::collections::BTreeMap::new();
        // Since we aren't tracking actual routing efficiency right now, we just populate the map
        for venue in self.runtime.router.venues.values() {
            venue_analytics.insert(venue.venue_id.0, crate::analytics::VenueAnalytics::default());
        }

        Ok(BenchmarkReport {
            events_per_sec,
            trades_per_sec,
            symbols_processed: self.analytics.symbol_distribution.len() as u64,
            total_journal_bytes: total_bytes,
            replay_duration_us: elapsed_us,
            realized_pnl,
            unrealized_pnl,
            order_flow: self.analytics.clone(),
            venue_analytics,
            clearing_analytics: crate::analytics::ClearingAnalytics::default(),
            systemic_stress_score_ppm: 0,
            venue_fragmentation_score_ppm: 0,
            governance_intervention_score_ppm: 0,
            recovery_efficiency_score_ppm: 0,
            replay_integrity_score_ppm: 0,
            operational_stability_score_ppm: 0,
            experiment_replay_integrity_ppm: 0,
            systemic_variance_score_ppm: 0,
            liquidity_resilience_score_ppm: 0,
            replay_divergence_score_ppm: 0,
            infrastructure_stability_score_ppm: 0,
            systemic_recovery_score_ppm: 0,
            intervention_effectiveness_ppm: 0,
            insolvency_containment_ppm: 0,
            liquidity_restoration_ppm: 0,
            policy_stability_score_ppm: 0,
            contagion_suppression_score_ppm: 0,
            replay_certification_score_ppm: 0,
            policy_response_efficiency_ppm: 0,
            liquidity_recovery_duration_sequences: 0,
            systemic_containment_score_ppm: 0,
            benchmark_integrity_score_ppm: 0,
            lineage_consistency_score_ppm: 0,
            benchmark_trust_score_ppm: 0,
            invariant_compliance_score_ppm: 0,
            system_integrity_score_ppm: 0,
            behavioral_stability_score_ppm: 0,
            agent_fragmentation_score_ppm: 0,
            liquidity_retreat_intensity_ppm: 0,
            cascade_acceleration_score_ppm: 0,
            market_resilience_score_ppm: 0,
            reward_efficiency_score_ppm: 0,
            dataset_integrity_score_ppm: 0,
            policy_replay_divergence_ppm: 0,
            adaptive_recovery_score_ppm: 0,
            distributed_replay_integrity_score_ppm: 0,
            shard_certification_score_ppm: 0,
            distributed_parity_score_ppm: 0,
            replay_cluster_efficiency_ppm: 0,
            lineage_merge_integrity_ppm: 0,
            formal_determinism_score_ppm: 0,
            replay_equivalence_score_ppm: 0,
            invariant_stability_score_ppm: 0,
            distributed_parity_integrity_ppm: 0,
            certification_chain_integrity_ppm: 0,
            federation_consensus_integrity_ppm: 0,
            cross_cluster_equivalence_score_ppm: 0,
            replay_treaty_stability_ppm: 0,
            sovereign_partition_integrity_ppm: 0,
            federation_lineage_continuity_ppm: 0,
            queue_integrity_score_ppm: 0,
            execution_parity_score_ppm: 0,
            latency_stability_score_ppm: 0,
            slippage_consistency_score_ppm: 0,
            orderbook_reconstruction_score_ppm: 0,
            strategy_parity_score_ppm: 0,
            portfolio_integrity_score_ppm: 0,
            execution_efficiency_score_ppm: 0,
            inventory_stability_score_ppm: 0,
            pnl_replay_consistency_ppm: 0,
            market_reconstruction_score_ppm: 0,
            historical_replay_integrity_ppm: 0,
            timestamp_alignment_score_ppm: 0,
            microstructure_fidelity_score_ppm: 0,
            session_equivalence_score_ppm: 0,
            exposure_integrity_score_ppm: 0,
            margin_stability_score_ppm: 0,
            liquidation_parity_score_ppm: 0,
            stress_propagation_integrity_ppm: 0,
            risk_constraint_stability_ppm: 0,
            replay_throughput_score_ppm: 0,
            profiling_integrity_score_ppm: 0,
            memory_stability_score_ppm: 0,
            execution_efficiency_score_ppm_2e: 0,
            certification_overhead_score_ppm: 0,
        })
    }

    fn process_event_for_inspection(&mut self, event: &AstraEvent) -> Result<(), String> {
        if event.event_type == EventType::LimitOrderPlaced {
            let payload: LimitOrderPlacedPayload =
                deserialize_canonical(&event.payload).map_err(|e| e.to_string())?;

            self.analytics.total_orders += 1;
            *self
                .analytics
                .symbol_distribution
                .entry(payload.symbol.clone())
                .or_insert(0) += 1;

            let side_str = match payload.side {
                OrderSide::Bid => "BUY".to_string(),
                OrderSide::Ask => "SELL".to_string(),
            };

            let trace_id = self.timeline.next_trace_id();
            self.timeline.record_accepted(
                trace_id,
                event.sequence_id,
                payload.trader_id,
                payload.symbol.clone(),
                side_str,
                payload.quantity.0,
                payload.price.0,
            );

            let notional = payload.price.0.saturating_mul(payload.quantity.0 as i64);

            // Peek at Risk Validation WITHOUT mutating
            if self
                .runtime
                .risk_engine
                .validate_order(payload.trader_id, payload.quantity.0, notional)
                .is_err()
            {
                self.analytics.total_rejections += 1;
                let reject_trace_id = self.timeline.next_trace_id();
                self.timeline.record_reject(
                    reject_trace_id,
                    event.sequence_id,
                    payload.trader_id,
                    "RiskLimitExceeded".to_string(),
                );
                return Ok(());
            }

            // To know fills exactly without modifying runtime, we clone the LOB for this symbol and simulate the submit!
            // This is slightly expensive but this is an offline observability tool.
            let mut book = self
                .runtime
                .router
                .venues
                .values()
                .next()
                .and_then(|v| v.books.get(&payload.symbol))
                .cloned()
                .unwrap_or_else(|| LimitOrderBook::new(payload.symbol.clone()));

            let order_side = match payload.side {
                OrderSide::Bid => astra_lob::types::OrderSide::Bid,
                OrderSide::Ask => astra_lob::types::OrderSide::Ask,
            };
            let order = astra_lob::types::Order {
                order_id: payload.order_id,
                symbol: payload.symbol.clone(),
                trader_id: payload.trader_id,
                side: order_side,
                order_type: astra_lob::types::OrderType::Limit,
                price: payload.price,
                quantity: payload.quantity,
                remaining_quantity: payload.quantity,
                timestamp_ns: event.timestamp_ns,
                queue_position: Default::default(),
            };

            let events = book.submit(order);
            for ev in events {
                match ev {
                    OrderEvent::TradeExecuted(exec) => {
                        self.analytics.total_fills += 1;
                        if exec.liquidity_side == astra_lob::types::LiquiditySide::Maker {
                            self.analytics.maker_volume += exec.matched_quantity.0;
                        } else {
                            self.analytics.taker_volume += exec.matched_quantity.0;
                        }

                        // We record only once for the timeline if it's the taker side to avoid duplicates
                        if exec.liquidity_side == astra_lob::types::LiquiditySide::Taker {
                            let fill_trace_id = self.timeline.next_trace_id();
                            self.timeline.record_fill(
                                fill_trace_id,
                                event.sequence_id,
                                exec.resting_order_id, // approximation for maker_trader_id
                                exec.aggressive_order_id, // approximation for taker_trader_id
                                exec.symbol,
                                exec.matched_quantity.0,
                                exec.match_price.0,
                                exec.liquidity_side,
                            );

                            let snapshot = book.snapshot();
                            let scaled_midpoint = snapshot.best_bid.unwrap_or(exec.match_price).0 + snapshot.best_ask.unwrap_or(exec.match_price).0;

                            self.execution_quality.record_fill(
                                false,
                                exec.matched_quantity.0,
                                exec.match_price.0,
                                scaled_midpoint,
                                scaled_midpoint, // Post-trade midpoint placeholder
                            );
                        } else {
                            // Maker (passive) fill
                            self.execution_quality.record_fill(
                                true,
                                exec.matched_quantity.0,
                                exec.match_price.0,
                                0,
                                0,
                            );

                            let initial_ahead = exec.queue_position.initial_ahead_quantity;
                            self.execution_quality.record_queue_advancement(initial_ahead, exec.matched_quantity.0);
                        }
                    }
                    OrderEvent::Cancelled {
                        order_id: _,
                        symbol,
                        reason: _,
                    } => {
                        // Approximation: We don't have the cancelled quantity in the OrderEvent::Cancelled right now without looking it up.
                        // Assuming total initial quantity for now, but really need to lookup.
                        self.execution_quality.record_cancel(payload.quantity.0);

                        self.analytics.cancel_count += 1;
                        let cancel_trace_id = self.timeline.next_trace_id();
                        self.timeline
                            .record_cancel(cancel_trace_id, event.sequence_id, payload.trader_id, symbol);
                    }
                    _ => {}
                }
            }

            let update_trace_id = self.timeline.next_trace_id();
            self.timeline.record_update(
                update_trace_id,
                event.sequence_id,
                payload.symbol.clone(),
                book.orders.len(),
            );
        }
        Ok(())
    }
}
