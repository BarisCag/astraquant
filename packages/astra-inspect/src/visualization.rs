use crate::analytics::BenchmarkReport;
use crate::strategy_analytics::StrategyAnalyticsCollector;
use crate::timeline::ReplayTimeline;
use astra_lob::book::LimitOrderBook;
use std::io::Write;

pub fn export_csv<W: Write>(out: &mut W, timeline: &ReplayTimeline) -> std::io::Result<()> {
    writeln!(
        out,
        "TRACE_ID,SEQUENCE,EVENT_TYPE,TRADER_ID,SYMBOL,SIDE,QUANTITY,PRICE,REASON"
    )?;
    for ev in &timeline.events {
        match ev {
            crate::timeline::TimelineEvent::Accepted {
                execution_trace_id,
                sequence,
                trader_id,
                symbol,
                side,
                quantity,
                price,
            } => {
                writeln!(
                    out,
                    "{},{},ACCEPT,{},{},{},{},{},",
                    execution_trace_id, sequence, trader_id, symbol, side, quantity, price
                )?;
            }
            crate::timeline::TimelineEvent::Fill {
                execution_trace_id,
                sequence,
                maker_trader_id,
                taker_trader_id,
                symbol,
                quantity,
                price,
                liquidity_side,
            } => {
                if *liquidity_side == astra_lob::types::LiquiditySide::Taker {
                    writeln!(
                        out,
                        "{},{},FILL,{},{},TAKER,{},{},MAKER={}",
                        execution_trace_id, sequence, taker_trader_id, symbol, quantity, price, maker_trader_id
                    )?;
                }
            }
            crate::timeline::TimelineEvent::Update {
                execution_trace_id,
                sequence,
                symbol,
                queue_depth,
            } => {
                writeln!(out, "{},{},UPDATE,,{},,{},,", execution_trace_id, sequence, symbol, queue_depth)?;
            }
            crate::timeline::TimelineEvent::Reject {
                execution_trace_id,
                sequence,
                trader_id,
                reason,
            } => {
                writeln!(out, "{},{},REJECT,{},,,,,{}", execution_trace_id, sequence, trader_id, reason)?;
            }
            crate::timeline::TimelineEvent::Cancel {
                execution_trace_id,
                sequence,
                trader_id,
                symbol,
            } => {
                writeln!(out, "{},{},CANCEL,{},{},,,,", execution_trace_id, sequence, trader_id, symbol)?;
            }
        }
    }
    Ok(())
}

pub fn export_json<W: Write>(out: &mut W, report: &BenchmarkReport) -> std::io::Result<()> {
    // We use serde_json for deterministic output
    let json = serde_json::to_string_pretty(report).unwrap();
    writeln!(out, "{}", json)
}

pub fn export_mermaid_sequence<W: Write>(
    out: &mut W,
    timeline: &ReplayTimeline,
) -> std::io::Result<()> {
    writeln!(out, "sequenceDiagram")?;
    // Collect unique traders
    let mut traders = std::collections::BTreeSet::new();
    for ev in &timeline.events {
        match ev {
            crate::timeline::TimelineEvent::Accepted { trader_id, .. } => {
                traders.insert(*trader_id);
            }
            crate::timeline::TimelineEvent::Reject { trader_id, .. } => {
                traders.insert(*trader_id);
            }
            crate::timeline::TimelineEvent::Cancel { trader_id, .. } => {
                traders.insert(*trader_id);
            }
            _ => {}
        }
    }

    for t in &traders {
        writeln!(out, "    participant Trader{}", t)?;
    }
    writeln!(out, "    participant Risk")?;
    writeln!(out, "    participant LOB")?;
    writeln!(out, "    participant Portfolio")?;

    for ev in &timeline.events {
        match ev {
            crate::timeline::TimelineEvent::Accepted { trader_id, .. } => {
                writeln!(out, "    Trader{}->>Risk: Submit Order", trader_id)?;
                writeln!(out, "    Risk->>LOB: Accepted")?;
            }
            crate::timeline::TimelineEvent::Fill { .. } => {
                writeln!(out, "    LOB->>Portfolio: TradeExecuted")?;
            }
            crate::timeline::TimelineEvent::Reject {
                trader_id, reason, ..
            } => {
                writeln!(out, "    Trader{}->>Risk: Submit Order", trader_id)?;
                writeln!(
                    out,
                    "    Risk-->>Trader{}: Rejected ({})",
                    trader_id, reason
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn export_mermaid_routing_trace<W: Write>(
    out: &mut W,
    timeline: &ReplayTimeline,
) -> std::io::Result<()> {
    writeln!(out, "sequenceDiagram")?;
    writeln!(out, "    participant SOR")?;
    writeln!(out, "    participant Venue1")?;
    writeln!(out, "    participant Venue2")?;
    writeln!(out, "    participant Venue3")?;
    
    for ev in &timeline.events {
        if let crate::timeline::TimelineEvent::Reject { reason, .. } = ev {
            if reason.starts_with("Venue Offline") {
                writeln!(out, "    Venue->>SOR: VenueOffline")?;
                writeln!(out, "    SOR->>Venue: Reroute OrderFlow")?;
            }
        }
    }
    Ok(())
}

pub fn export_ascii_lob<W: Write>(out: &mut W, book: &LimitOrderBook) -> std::io::Result<()> {
    writeln!(out, "ASKS")?;
    // Asks are sorted ascending, we want to print highest ask first down to best ask
    for (price, level) in book.asks.iter().rev() {
        let total_quantity = level
            .orders
            .iter()
            .map(|o| o.remaining_quantity.0)
            .sum::<u64>();
        let mut queue_str = String::new();
        for order in &level.orders {
            queue_str.push_str(&format!("[T{}:{}] ", order.trader_id, order.remaining_quantity.0));
        }
        let hashes = "#".repeat(std::cmp::min(total_quantity, 10) as usize);
        writeln!(out, "{} | {}({}) | Q: {}", price.0, hashes, total_quantity, queue_str)?;
    }

    let best_ask = book.asks.keys().next();
    let best_bid = book.bids.keys().next_back();

    if let (Some(a), Some(b)) = (best_ask, best_bid) {
        let spread = a.0 - b.0;
        writeln!(out, "\nSPREAD = {}\n", spread)?;
    } else {
        writeln!(out, "\nSPREAD = INF\n")?;
    }

    // Bids are sorted ascending, we want best bid (highest) down to lowest
    for (price, level) in book.bids.iter().rev() {
        let total_quantity = level
            .orders
            .iter()
            .map(|o| o.remaining_quantity.0)
            .sum::<u64>();
        let mut queue_str = String::new();
        for order in &level.orders {
            queue_str.push_str(&format!("[T{}:{}] ", order.trader_id, order.remaining_quantity.0));
        }
        let hashes = "#".repeat(std::cmp::min(total_quantity, 10) as usize);
        writeln!(out, "{} | {}({}) | Q: {}", price.0, hashes, total_quantity, queue_str)?;
    }
    writeln!(out, "BIDS")?;
    Ok(())
}

pub fn export_multi_venue_ascii_lob<W: Write>(out: &mut W, venues: &std::collections::BTreeMap<astra_router::venue::VenueId, astra_router::venue::VenueState>, symbol: &str) -> std::io::Result<()> {
    writeln!(out, "==============================================")?;
    writeln!(out, "   MULTI-VENUE LIQUIDITY HEATMAP: {}", symbol)?;
    writeln!(out, "==============================================")?;
    
    for venue in venues.values() {
        writeln!(out, "\nVENUE ID: {}", venue.venue_id.0)?;
        if let Some(book) = venue.books.get(symbol) {
            export_ascii_lob(out, book)?;
        } else {
            writeln!(out, "No liquidity for symbol.")?;
        }
    }
    Ok(())
}

pub fn export_mermaid_queue_evolution<W: Write>(
    out: &mut W,
    timeline: &ReplayTimeline,
) -> std::io::Result<()> {
    writeln!(out, "sequenceDiagram")?;
    writeln!(out, "    participant Market")?;
    writeln!(out, "    participant Queue")?;
    
    for ev in &timeline.events {
        match ev {
            crate::timeline::TimelineEvent::Accepted { execution_trace_id, quantity, price, side, .. } => {
                writeln!(out, "    Market->>Queue: T{} {} {} @ {}", execution_trace_id, side, quantity, price)?;
            }
            crate::timeline::TimelineEvent::Fill { execution_trace_id, quantity, price, liquidity_side, .. } => {
                let liq = if *liquidity_side == astra_lob::types::LiquiditySide::Maker { "Maker" } else { "Taker" };
                writeln!(out, "    Queue->>Market: T{} Filled {} @ {} ({})", execution_trace_id, quantity, price, liq)?;
            }
            crate::timeline::TimelineEvent::Cancel { execution_trace_id, symbol, .. } => {
                writeln!(out, "    Queue->>Market: T{} Cancelled ({})", execution_trace_id, symbol)?;
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn export_mermaid_strategy_trace<W: Write>(
    out: &mut W,
    collector: &StrategyAnalyticsCollector,
) -> std::io::Result<()> {
    writeln!(out, "gantt")?;
    writeln!(out, "    title Strategy Inventory Trace")?;
    writeln!(out, "    dateFormat  x")?;
    writeln!(out, "    axisFormat %s")?;

    for (trader_id, metrics) in &collector.metrics_by_trader {
        writeln!(out, "    section Trader {}", trader_id)?;
        let mut prev_ts: Option<u64> = None;
        let mut prev_inv: Option<i64> = None;

        for (&ts, &inv) in &metrics.inventory_series {
            if let (Some(pts), Some(pinv)) = (prev_ts, prev_inv) {
                if pinv != 0 {
                    writeln!(out, "    Inventory {} : {} , {}", pinv, pts, ts)?;
                }
            }
            prev_ts = Some(ts);
            prev_inv = Some(inv);
        }
    }
    Ok(())
}

pub fn export_strategy_analytics_json<W: Write>(
    out: &mut W,
    collector: &StrategyAnalyticsCollector,
) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(&collector.metrics_by_trader).unwrap();
    writeln!(out, "{}", json)
}

pub fn export_liquidation_cascade_timeline<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD\n    A[Margin Breach] --> B[Forced Liquidation]")
}

pub fn export_venue_failure_topology_map<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD\n    SOR --> V1[Venue 1 - FAILED]")
}

pub fn export_collateral_deterioration_heatmap<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "Collateral Heatmap Placeholder")
}

pub fn export_funding_imbalance_diagram<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "Funding Imbalance Placeholder")
}

pub fn export_operational_intervention_timeline<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD\n    A[Intervention Start] --> B[Venue Paused]")
}

pub fn export_replay_divergence_map<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD\n    A[Checkpoint Match] --> B[Divergence Detected]")
}

pub fn export_recovery_topology_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "Recovery Topology Placeholder")
}

pub fn export_checkpoint_lineage_diagram<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "Checkpoint Lineage Placeholder")
}

pub fn export_governance_decision_tree<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "Governance Decision Tree Placeholder")
}

pub fn export_replay_checkpoint_diff<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "Checkpoint Diff Placeholder")
}

pub fn export_experiment_comparison_heatmap<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "Experiment Comparison Heatmap Placeholder")
}

pub fn export_replay_divergence_tree<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "Replay Divergence Tree Placeholder")
}

pub fn export_parameter_sweep_topology_map<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "Parameter Sweep Topology Map Placeholder")
}

pub fn export_scenario_branching_diagram<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "Scenario Branching Diagram Placeholder")
}

pub fn export_recovery_policy_comparison_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "Recovery Policy Comparison Graph Placeholder")
}

// Phase 13C: Deterministic Audit Visualizations

pub fn export_replay_lineage_tree<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    ROOT[\"Root Hash\"] --> CP1[\"Checkpoint 1000\"]")?;
    writeln!(out, "    CP1 --> CP2[\"Checkpoint 2000\"]")?;
    writeln!(out, "    CP2 --> TERMINAL[\"Terminal Hash\"]")?;
    writeln!(out, "    style ROOT fill:#2d3748,color:#e2e8f0")?;
    writeln!(out, "    style TERMINAL fill:#2f855a,color:#e2e8f0")
}

pub fn export_invariant_violation_map<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    SEQ_MON[\"Sequence Monotonicity\"] -->|PASS| OK1((✓))")?;
    writeln!(out, "    JOURNAL[\"Journal Continuity\"] -->|PASS| OK2((✓))")?;
    writeln!(out, "    REPLAY[\"Replay Parity\"] -->|PASS| OK3((✓))")?;
    writeln!(out, "    LIQUIDITY[\"Liquidity Conservation\"] -->|FAIL| FAIL1((✗))")?;
    writeln!(out, "    style FAIL1 fill:#c53030,color:#fff")
}

pub fn export_replay_divergence_topology<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    COMMON[\"Common Ancestor\"] --> LEFT[\"Left Branch\"]")?;
    writeln!(out, "    COMMON --> RIGHT[\"Right Branch\"]")?;
    writeln!(out, "    LEFT --> L_TERM[\"Left Terminal\"]")?;
    writeln!(out, "    RIGHT --> R_TERM[\"Right Terminal\"]")?;
    writeln!(out, "    style COMMON fill:#2b6cb0,color:#e2e8f0")
}

pub fn export_certification_ancestry_diagram<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph BT")?;
    writeln!(out, "    CERT[\"Terminal Certificate\"] --> PROOF[\"Parity Proof\"]")?;
    writeln!(out, "    PROOF --> LINEAGE[\"Lineage Tree\"]")?;
    writeln!(out, "    LINEAGE --> MANIFEST[\"Benchmark Manifest\"]")?;
    writeln!(out, "    style CERT fill:#2f855a,color:#e2e8f0")
}

pub fn export_benchmark_trust_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    REPLAY[\"Replay Integrity\"] --> TRUST[\"Overall Trust\"]")?;
    writeln!(out, "    LINEAGE[\"Lineage Consistency\"] --> TRUST")?;
    writeln!(out, "    INVARIANT[\"Invariant Compliance\"] --> TRUST")?;
    writeln!(out, "    BENCHMARK[\"Benchmark Integrity\"] --> TRUST")?;
    writeln!(out, "    style TRUST fill:#2f855a,color:#e2e8f0")
}

// Phase 14A: Ecology Visualizations

pub fn export_agent_topology_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    MM[Market Maker] -->|Provides| VENUE[Exchange Venue]")?;
    writeln!(out, "    ARB[Arbitrageur] -->|Balances| VENUE")?;
    writeln!(out, "    LIQ[Panic Liquidator] -->|Drains| VENUE")?;
    writeln!(out, "    style VENUE fill:#2b6cb0,color:#fff")
}

pub fn export_liquidity_retreat_tree<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    STRESS[Volatility Spike] --> MM_W[MM Withdrawal]")?;
    writeln!(out, "    MM_W --> SPREAD[Spread Widening]")?;
    writeln!(out, "    SPREAD --> PANIC[Panic Liquidation]")?;
    writeln!(out, "    style STRESS fill:#c53030,color:#fff")
}

pub fn export_systemic_panic_topology<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    P1[Distressed Fund] --> M1[Margin Call]")?;
    writeln!(out, "    M1 --> L1[Forced Liquidation]")?;
    writeln!(out, "    L1 --> P2[Clearing Member Stress]")?;
    writeln!(out, "    style P1 fill:#dd6b20,color:#fff")
}

// Phase 14B: RL Visualizations

pub fn export_policy_transition_topology<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    S1[Normal] -->|Widen Spread| S2[Stress]")?;
    writeln!(out, "    S2 -->|Reduce Inventory| S3[Recovery]")?;
    writeln!(out, "    style S1 fill:#3182ce,color:#fff")
}

pub fn export_reward_evolution_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    E1[Epoch 1] --> E2[Epoch 2]")?;
    writeln!(out, "    E2 --> E3[Epoch 3]")?;
    writeln!(out, "    style E3 fill:#38a169,color:#fff")
}

pub fn export_trajectory_lineage_tree<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    ROOT[Root State] --> T1[Trajectory A]")?;
    writeln!(out, "    ROOT --> T2[Trajectory B]")?;
    writeln!(out, "    style ROOT fill:#805ad5,color:#fff")
}

pub fn export_adaptive_containment_topology<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    PANIC[Panic] --> ACT[Policy Action]")?;
    writeln!(out, "    ACT --> CONT[Containment]")?;
    writeln!(out, "    style CONT fill:#38a169,color:#fff")
}

pub fn export_policy_divergence_map<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    STATE[State] -->|Policy A| A[Action A]")?;
    writeln!(out, "    STATE -->|Policy B| B[Action B]")?;
    writeln!(out, "    style STATE fill:#718096,color:#fff")
}

// Phase 15A: Distributed Replay Visualizations

pub fn export_replay_shard_topology<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    M[Manifest] --> S1[Shard 1]")?;
    writeln!(out, "    M --> S2[Shard 2]")?;
    writeln!(out, "    style M fill:#2b6cb0,color:#fff")
}

pub fn export_distributed_execution_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    O[Orchestrator] -->|Dispatch| W1[Worker A]")?;
    writeln!(out, "    O -->|Dispatch| W2[Worker B]")?;
    writeln!(out, "    style O fill:#c53030,color:#fff")
}

pub fn export_certification_aggregation_tree<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph BT")?;
    writeln!(out, "    C1[Cert 1] --> ROOT[Aggregated Cert]")?;
    writeln!(out, "    C2[Cert 2] --> ROOT")?;
    writeln!(out, "    style ROOT fill:#38a169,color:#fff")
}

pub fn export_replay_lineage_merge_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    L1[Lineage A] --> M[Merge Node]")?;
    writeln!(out, "    L2[Lineage B] --> M")?;
    writeln!(out, "    M --> FINAL[Global Lineage]")?;
    writeln!(out, "    style FINAL fill:#805ad5,color:#fff")
}

pub fn export_distributed_benchmark_fanout<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    B[Benchmark] -->|Fanout| C1[Cluster 1]")?;
    writeln!(out, "    B -->|Fanout| C2[Cluster 2]")?;
    writeln!(out, "    style B fill:#dd6b20,color:#fff")
}

// Phase 16A: Formal Verification Visualizations

pub fn export_invariant_dependency_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    I1[Invariant A] --> I2[Invariant B]")?;
    writeln!(out, "    style I1 fill:#2b6cb0,color:#fff")
}

pub fn export_replay_proof_chain<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    P1[Proof 1] --> P2[Proof 2]")?;
    writeln!(out, "    style P2 fill:#38a169,color:#fff")
}

pub fn export_lineage_certification_topology<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    ROOT[Root Node] --> L1[Lineage Node]")?;
    writeln!(out, "    style ROOT fill:#805ad5,color:#fff")
}

pub fn export_aggregation_parity_tree<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph BT")?;
    writeln!(out, "    P1[Parity 1] --> ROOT[Aggregated Parity]")?;
    writeln!(out, "    style ROOT fill:#e53e3e,color:#fff")
}

pub fn export_distributed_equivalence_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    MON[Monolithic] <--> DIST[Distributed]")?;
    writeln!(out, "    style DIST fill:#d69e2e,color:#fff")
}

// Phase 17A: Federation Visualizations

pub fn export_federation_topology_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    F1[Federation Cluster 1] <--> F2[Federation Cluster 2]")?;
    writeln!(out, "    style F1 fill:#2c5282,color:#fff")
}

pub fn export_cross_cluster_lineage_bridge<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    C1[Cluster 1 Lineage] -->|Bridge| C2[Cluster 2 Lineage]")?;
    writeln!(out, "    style C2 fill:#276749,color:#fff")
}

pub fn export_replay_treaty_dependency_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    T[Treaty] --> R1[Requirement 1]")?;
    writeln!(out, "    style T fill:#9b2c2c,color:#fff")
}

pub fn export_sovereign_partition_map<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph BT")?;
    writeln!(out, "    P1[Partition 1] --> S[Sovereign Domain]")?;
    writeln!(out, "    style S fill:#b83280,color:#fff")
}

pub fn export_federated_certification_mesh<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    C1[Cert A] <--> C2[Cert B]")?;
    writeln!(out, "    style C1 fill:#b7791f,color:#fff")
}

// Stage 2A: LOB Hardening Visualizations

pub fn export_queue_evolution_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    Q1[Queue State 1] --> Q2[Queue State 2]")?;
    writeln!(out, "    style Q1 fill:#2b6cb0,color:#fff")
}

pub fn export_fill_lineage_topology<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    F1[Fill 1] --> F2[Fill 2]")?;
    writeln!(out, "    style F2 fill:#38a169,color:#fff")
}

pub fn export_orderbook_depth_evolution<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    D1[Depth 1] --> D2[Depth 2]")?;
    writeln!(out, "    style D1 fill:#d69e2e,color:#fff")
}

pub fn export_latency_propagation_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    E1[Event] -->|Latency| E2[Arrival]")?;
    writeln!(out, "    style E2 fill:#e53e3e,color:#fff")
}

pub fn export_execution_pressure_topology<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph BT")?;
    writeln!(out, "    P1[Pressure] --> P2[Higher Pressure]")?;
    writeln!(out, "    style P2 fill:#805ad5,color:#fff")
}

// Stage 2B: Strategy & Portfolio Visualizations

pub fn export_strategy_execution_topology<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    S[Signal] --> I[Intent] --> E[Execution]")?;
    writeln!(out, "    style S fill:#3182ce,color:#fff")
}

pub fn export_pnl_evolution_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    P1[PnL State 1] --> P2[PnL State 2]")?;
    writeln!(out, "    style P2 fill:#38a169,color:#fff")
}

pub fn export_inventory_exposure_topology<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    I1[Inventory A] --> I2[Inventory B]")?;
    writeln!(out, "    style I1 fill:#d69e2e,color:#fff")
}

pub fn export_trade_lifecycle_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    O[Order] --> F[Fill] --> P[Position]")?;
    writeln!(out, "    style P fill:#e53e3e,color:#fff")
}

pub fn export_execution_efficiency_map<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph BT")?;
    writeln!(out, "    E1[Efficiency] --> E2[Higher Efficiency]")?;
    writeln!(out, "    style E2 fill:#805ad5,color:#fff")
}

// Stage 2C: Historical Reconstruction Visualizations

pub fn export_market_session_topology<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    S1[Session Block 1] --> S2[Session Block 2]")?;
    writeln!(out, "    style S1 fill:#319795,color:#fff")
}

pub fn export_replay_lineage_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    L1[Lineage Node A] --> L2[Lineage Node B]")?;
    writeln!(out, "    style L2 fill:#dd6b20,color:#fff")
}

pub fn export_venue_fragmentation_evolution<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    V[Global Orderbook] --> V1[Venue A]")?;
    writeln!(out, "    V --> V2[Venue B]")?;
    writeln!(out, "    style V fill:#718096,color:#fff")
}

pub fn export_liquidity_regime_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    R1[Low Volatility] --> R2[High Volatility]")?;
    writeln!(out, "    style R2 fill:#e53e3e,color:#fff")
}

pub fn export_timestamp_alignment_topology<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph BT")?;
    writeln!(out, "    T1[Raw Time] --> T2[Aligned Time]")?;
    writeln!(out, "    style T2 fill:#3182ce,color:#fff")
}

// Stage 2D: Risk, Margin & Exposure Visualizations

pub fn export_exposure_topology<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    E1[Exposure Node 1] --> E2[Exposure Node 2]")?;
    writeln!(out, "    style E1 fill:#dd6b20,color:#fff")
}

pub fn export_liquidation_cascade_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    L1[Liquidation Event] --> L2[Cascade Target]")?;
    writeln!(out, "    style L2 fill:#e53e3e,color:#fff")
}

pub fn export_collateral_exhaustion_tree<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    C1[Collateral Buffer] --> C2[Exhaustion Node]")?;
    writeln!(out, "    style C2 fill:#805ad5,color:#fff")
}

pub fn export_systemic_stress_propagation_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    S1[Shock Origin] --> S2[Contagion Node]")?;
    writeln!(out, "    style S2 fill:#c53030,color:#fff")
}

pub fn export_risk_constraint_dependency_map<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph BT")?;
    writeln!(out, "    C1[Constraint] --> P[Portfolio Strategy]")?;
    writeln!(out, "    style C1 fill:#b7791f,color:#fff")
}

// Stage 2E: Performance & Profiling Visualizations

pub fn export_replay_throughput_topology<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    T1[Batch Start] --> T2[Batch Processed]")?;
    writeln!(out, "    style T2 fill:#38a169,color:#fff")
}

pub fn export_execution_batch_evolution<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    B1[Batch N] --> B2[Batch N+1]")?;
    writeln!(out, "    style B2 fill:#3182ce,color:#fff")
}

pub fn export_memory_pressure_topology<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph TD")?;
    writeln!(out, "    M1[Allocation] --> M2[Pressure Boundary]")?;
    writeln!(out, "    style M2 fill:#e53e3e,color:#fff")
}

pub fn export_certification_overhead_graph<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph BT")?;
    writeln!(out, "    O1[Base Overhead] --> O2[Certification Hash]")?;
    writeln!(out, "    style O2 fill:#805ad5,color:#fff")
}

pub fn export_replay_scaling_trace<W: Write>(out: &mut W) -> std::io::Result<()> {
    writeln!(out, "graph LR")?;
    writeln!(out, "    S1[Node 1] --> S2[Node 2]")?;
    writeln!(out, "    style S2 fill:#dd6b20,color:#fff")
}
