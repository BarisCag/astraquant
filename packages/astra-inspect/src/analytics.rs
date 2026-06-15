use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OrderFlowAnalytics {
    pub total_orders: u64,
    pub total_fills: u64,
    pub total_rejections: u64,
    pub maker_volume: u64,
    pub taker_volume: u64,
    pub cancel_count: u64,
    pub symbol_distribution: BTreeMap<String, u64>,
}

impl OrderFlowAnalytics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fill_ratio_ppm(&self) -> u64 {
        if self.total_orders == 0 {
            return 0;
        }
        (self.total_fills.saturating_mul(1_000_000)) / self.total_orders
    }

    pub fn rejection_ratio_ppm(&self) -> u64 {
        if self.total_orders == 0 {
            return 0;
        }
        (self.total_rejections.saturating_mul(1_000_000)) / self.total_orders
    }

    pub fn maker_ratio_ppm(&self) -> u64 {
        let total = self.maker_volume.saturating_add(self.taker_volume);
        if total == 0 {
            return 0;
        }
        (self.maker_volume.saturating_mul(1_000_000)) / total
    }

    pub fn taker_ratio_ppm(&self) -> u64 {
        let total = self.maker_volume.saturating_add(self.taker_volume);
        if total == 0 {
            return 0;
        }
        (self.taker_volume.saturating_mul(1_000_000)) / total
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub events_per_sec: u64,
    pub trades_per_sec: u64,
    pub symbols_processed: u64,
    pub total_journal_bytes: u64,
    pub replay_duration_us: u64,
    pub realized_pnl: i64,
    pub unrealized_pnl: i64,
    pub order_flow: OrderFlowAnalytics,
    pub venue_analytics: BTreeMap<u8, VenueAnalytics>,
    pub clearing_analytics: ClearingAnalytics,
    pub systemic_stress_score_ppm: u64,
    pub venue_fragmentation_score_ppm: u64,
    pub governance_intervention_score_ppm: u64,
    pub recovery_efficiency_score_ppm: u64,
    pub replay_integrity_score_ppm: u64,
    pub operational_stability_score_ppm: u64,

    // Experiment Level Metrics
    pub experiment_replay_integrity_ppm: u64,
    pub systemic_variance_score_ppm: u64,
    pub liquidity_resilience_score_ppm: u64,
    pub replay_divergence_score_ppm: u64,
    pub infrastructure_stability_score_ppm: u64,

    // Phase 13A Policy Metrics
    pub systemic_recovery_score_ppm: u64,
    pub intervention_effectiveness_ppm: u64,
    pub insolvency_containment_ppm: u64,
    pub liquidity_restoration_ppm: u64,
    pub policy_stability_score_ppm: u64,
    pub contagion_suppression_score_ppm: u64,

    // Phase 13B Benchmark Metrics
    pub replay_certification_score_ppm: u64,
    pub policy_response_efficiency_ppm: u64,
    pub liquidity_recovery_duration_sequences: u64,
    pub systemic_containment_score_ppm: u64,
    pub benchmark_integrity_score_ppm: u64,

    // Phase 13C Audit & Trust Metrics
    pub lineage_consistency_score_ppm: u64,
    pub benchmark_trust_score_ppm: u64,
    pub invariant_compliance_score_ppm: u64,
    pub system_integrity_score_ppm: u64,

    // Phase 14A Ecology Metrics
    pub behavioral_stability_score_ppm: u64,
    pub agent_fragmentation_score_ppm: u64,
    pub liquidity_retreat_intensity_ppm: u64,
    pub cascade_acceleration_score_ppm: u64,
    pub market_resilience_score_ppm: u64,

    // Phase 14B RL Telemetry Metrics
    pub reward_efficiency_score_ppm: u64,
    pub dataset_integrity_score_ppm: u64,
    pub policy_replay_divergence_ppm: u64,
    pub adaptive_recovery_score_ppm: u64,

    // Phase 15A Distributed Telemetry Metrics
    pub distributed_replay_integrity_score_ppm: u64,
    pub shard_certification_score_ppm: u64,
    pub distributed_parity_score_ppm: u64,
    pub replay_cluster_efficiency_ppm: u64,
    pub lineage_merge_integrity_ppm: u64,

    // Phase 16A Formal Telemetry Metrics
    pub formal_determinism_score_ppm: u64,
    pub replay_equivalence_score_ppm: u64,
    pub invariant_stability_score_ppm: u64,
    pub distributed_parity_integrity_ppm: u64,
    pub certification_chain_integrity_ppm: u64,

    // Phase 17A Federated Telemetry Metrics
    pub federation_consensus_integrity_ppm: u64,
    pub cross_cluster_equivalence_score_ppm: u64,
    pub replay_treaty_stability_ppm: u64,
    pub sovereign_partition_integrity_ppm: u64,
    pub federation_lineage_continuity_ppm: u64,

    // Stage 2A LOB Hardening Metrics
    pub queue_integrity_score_ppm: u64,
    pub execution_parity_score_ppm: u64,
    pub latency_stability_score_ppm: u64,
    pub slippage_consistency_score_ppm: u64,
    pub orderbook_reconstruction_score_ppm: u64,

    // Stage 2B Deterministic Strategy Metrics
    pub strategy_parity_score_ppm: u64,
    pub portfolio_integrity_score_ppm: u64,
    pub execution_efficiency_score_ppm: u64,
    pub inventory_stability_score_ppm: u64,
    pub pnl_replay_consistency_ppm: u64,

    // Stage 2C Historical Reconstruction Metrics
    pub market_reconstruction_score_ppm: u64,
    pub historical_replay_integrity_ppm: u64,
    pub timestamp_alignment_score_ppm: u64,
    pub microstructure_fidelity_score_ppm: u64,
    pub session_equivalence_score_ppm: u64,

    // Stage 2D Risk & Exposure Metrics
    pub exposure_integrity_score_ppm: u64,
    pub margin_stability_score_ppm: u64,
    pub liquidation_parity_score_ppm: u64,
    pub stress_propagation_integrity_ppm: u64,
    pub risk_constraint_stability_ppm: u64,

    // Stage 2E Performance & Profiling Metrics
    pub replay_throughput_score_ppm: u64,
    pub profiling_integrity_score_ppm: u64,
    pub memory_stability_score_ppm: u64,
    pub execution_efficiency_score_ppm_2e: u64,
    pub certification_overhead_score_ppm: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VenueAnalytics {
    pub total_fills: u64,
    pub volume_executed: u64,
    pub routing_efficiency_bps: i64,
    pub toxicity_bps: i64,
    pub post_trade_markout: i64,
    pub liquidity_survival_rate_ppm: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ClearingAnalytics {
    pub collateral_utilization_ppm: u64,
    pub settlement_failure_count: u64,
    pub liquidation_count: u64,
    pub margin_utilization_ppm: u64,
    pub funding_imbalance: i64,
}
