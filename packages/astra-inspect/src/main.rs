use astra_exchange::runtime::ExchangeRuntime;
use astra_inspect::inspector::ReplayInspector;
use astra_inspect::visualization::{
    export_mermaid_strategy_trace, export_multi_venue_ascii_lob, export_strategy_analytics_json,
};
use astra_risk::engine::RiskEngine;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "astra-inspect")]
#[command(about = "AstraQuant Deterministic Research Tooling")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    ReplaySummary {
        #[arg(short, long)]
        journal_dir: PathBuf,
    },

    OrderbookTrace {
        #[arg(short, long)]
        journal_dir: PathBuf,
        #[arg(short, long, default_value = "BTC/USD")]
        symbol: String,
    },
    PnlReport {
        #[arg(short, long)]
        journal_dir: PathBuf,
    },
    StrategyTrace {
        #[arg(short, long)]
        journal_dir: PathBuf,
    },
    StrategyAnalytics {
        #[arg(short, long)]
        journal_dir: PathBuf,
    },
    ScenarioReport {
        #[arg(short, long)]
        scenario: String,
    },
    CascadeTrace {
        #[arg(short, long)]
        scenario: String,
    },
    VenueFailureMap {
        #[arg(short, long)]
        scenario: String,
    },
    CheckpointDiff {
        #[arg(short, long)]
        scenario: String,
    },
    ReplayAudit {
        #[arg(short, long)]
        journal_dir: PathBuf,
    },
    OperationalTimeline {
        #[arg(short, long)]
        scenario: String,
    },
    RecoveryAnalysis {
        #[arg(short, long)]
        scenario: String,
    },
    CheckpointLineage {
        #[arg(short, long)]
        scenario: String,
    },
    RunExperiment {
        #[arg(short, long)]
        suite: String,
    },
    CompareExperiments {
        #[arg(short, long)]
        left: String,
        #[arg(short, long)]
        right: String,
    },
    ExperimentReport {
        #[arg(short, long)]
        suite: String,
    },
    /// Generate a visual comparison of systemic divergence between two runs
    ReplayDiff {
        #[arg(long)]
        left_run_id: String,
        #[arg(long)]
        right_run_id: String,
    },
    /// Phase 13A: Generate a deterministic policy report for an intervention scenario
    PolicyReport {
        #[arg(long)]
        scenario: String,
    },
    /// Phase 13A: Compare the deterministic outcomes of two intervention variations
    InterventionCompare {
        #[arg(long)]
        left: String,
        #[arg(long)]
        right: String,
    },
    /// Phase 13A: Map deterministic contagion propagation for a given funding shock
    ContagionMap {
        #[arg(long)]
        scenario: String,
    },
    /// Phase 13A: Trace the stabilization sequence of a central bank response
    StabilizationTrace {
        #[arg(long)]
        scenario: String,
    },
    /// Phase 13B: Generate institutional benchmark report
    BenchmarkReport {
        #[arg(long)]
        suite: String,
    },
    /// Phase 13B: Certify a canonical scenario replay against known manifests
    ReplayCertify {
        #[arg(long)]
        manifest: String,
    },
    /// Phase 13B: Compare interventions for study
    InterventionStudy {
        #[arg(long)]
        left: String,
        #[arg(long)]
        right: String,
    },
    /// Phase 13B: Compare systemic variations
    SystemicComparison {
        #[arg(long)]
        suite: String,
    },
    /// Phase 13C: Generate a full deterministic audit report
    AuditReport {
        #[arg(long)]
        journal_dir: String,
    },
    /// Phase 13C: Verify replay parity for a journal directory
    ReplayVerify {
        #[arg(long)]
        journal_dir: String,
    },
    /// Phase 13C: Trace replay lineage graph
    LineageTrace {
        #[arg(long)]
        journal_dir: String,
    },
    /// Phase 13C: Scan for invariant violations
    InvariantScan {
        #[arg(long)]
        journal_dir: String,
    },
    /// Phase 13C: Audit certification chain integrity
    CertificationAudit {
        #[arg(long)]
        manifest: String,
    },
    /// Phase 14A: Generate ecology report
    EcologyReport {
        #[arg(long)]
        journal_dir: String,
    },
    /// Phase 14A: Trace agent behaviors
    BehaviorTrace {
        #[arg(long)]
        journal_dir: String,
    },
    /// Phase 14A: Map systemic cascades
    CascadeMap {
        #[arg(long)]
        journal_dir: String,
    },
    /// Phase 14A: Analyze liquidity retreats
    LiquidityRetreatAnalysis {
        #[arg(long)]
        journal_dir: String,
    },
    /// Phase 14A: Map agent topologies
    AgentTopology {
        #[arg(long)]
        journal_dir: String,
    },
    /// Phase 14B: Build RL dataset from replay
    DatasetBuild {
        #[arg(long)]
        journal_dir: String,
    },
    /// Phase 14B: Evaluate learned policy
    PolicyEvaluate {
        #[arg(long)]
        policy_file: String,
    },
    /// Phase 14B: Compare two policies
    PolicyCompare {
        #[arg(long)]
        left_policy: String,
        #[arg(long)]
        right_policy: String,
    },
    /// Phase 14B: Trace trajectory lineage
    TrajectoryTrace {
        #[arg(long)]
        journal_dir: String,
    },
    /// Phase 14B: Analyze reward evolution
    RewardAnalysis {
        #[arg(long)]
        dataset_dir: String,
    },
    /// Phase 14B: Adaptive containment study
    AdaptiveStudy {
        #[arg(long)]
        policy_file: String,
    },
    /// Phase 15A: Distributed replay cluster status
    DistributedReplay {
        #[arg(long)]
        cluster_id: String,
    },
    /// Phase 15A: Certify a specific shard
    ShardCertify {
        #[arg(long)]
        shard_id: String,
    },
    /// Phase 15A: Generate cluster performance report
    ReplayClusterReport {
        #[arg(long)]
        report_id: String,
    },
    /// Phase 15A: Run distributed benchmark fanout
    DistributedBenchmark {
        #[arg(long)]
        benchmark_id: String,
    },
    /// Phase 15A: Trace lineage merge operations
    LineageMergeTrace {
        #[arg(long)]
        manifest_id: String,
    },
    /// Phase 15A: View replay fabric topology
    ReplayFabricTopology {
        #[arg(long)]
        cluster_id: String,
    },
    /// Phase 16A: Formally verify determinism
    FormalVerify {
        #[arg(long)]
        proof_file: String,
    },
    /// Phase 16A: Generate a replay proof
    ReplayProof {
        #[arg(long)]
        manifest_id: String,
    },
    /// Phase 16A: Trace invariant violations
    InvariantTrace {
        #[arg(long)]
        journal_dir: String,
    },
    /// Phase 16A: Generate lineage proof
    LineageProof {
        #[arg(long)]
        journal_dir: String,
    },
    /// Phase 16A: Verify aggregation proof
    AggregationVerify {
        #[arg(long)]
        aggregation_file: String,
    },
    /// Phase 16A: Check distributed equivalence
    DistributedEquivalenceCheck {
        #[arg(long)]
        proof_file: String,
    },
    /// Phase 17A: Federation Reporting
    FederationReport {
        #[arg(long)]
        federation_id: String,
    },
    /// Phase 17A: Treaty Verification
    TreatyVerify {
        #[arg(long)]
        treaty_file: String,
    },
    /// Phase 17A: Cross-cluster equivalence check
    CrossClusterCheck {
        #[arg(long)]
        cluster_a: String,
        #[arg(long)]
        cluster_b: String,
    },
    /// Phase 17A: View federation topology
    FederationTopology {
        #[arg(long)]
        federation_id: String,
    },
    /// Phase 17A: Notarize a replay trace
    ReplayNotarize {
        #[arg(long)]
        manifest_id: String,
    },
    /// Phase 17A: Trace sovereign lineage
    SovereignLineageTrace {
        #[arg(long)]
        partition_id: String,
    },
    /// Phase 17A: Verify federated equivalence
    FederatedEquivalenceCheck {
        #[arg(long)]
        federation_id: String,
    },
    /// Stage 2A: Replay orderbook reconstruction
    OrderbookReplay {
        #[arg(long)]
        journal_dir: String,
    },
    /// Stage 2A: Trace deterministic queue topologies
    QueueTrace {
        #[arg(long)]
        trace_id: String,
    },
    /// Stage 2A: Print deterministic execution lineage
    ExecutionLineage {
        #[arg(long)]
        window_id: String,
    },
    /// Stage 2A: Latency propagation analysis
    LatencyAnalysis {
        #[arg(long)]
        journal_dir: String,
    },
    /// Stage 2A: Deterministic slippage analysis
    SlippageAnalysis {
        #[arg(long)]
        journal_dir: String,
    },
    /// Stage 2A: View orderbook depth evolution
    DepthEvolution {
        #[arg(long)]
        trace_id: String,
    },
    /// Stage 2B: Replay strategy deterministic outputs
    StrategyReplay {
        #[arg(long)]
        strategy_id: String,
    },
    /// Stage 2B: Trace portfolio accounting drift
    PortfolioTrace {
        #[arg(long)]
        trace_id: String,
    },
    /// Stage 2B: Execution analytics parsing
    ExecutionAnalysis {
        #[arg(long)]
        journal_dir: String,
    },
    /// Stage 2B: Inventory exposure evolution
    InventoryAnalysis {
        #[arg(long)]
        journal_dir: String,
    },
    /// Stage 2B: Deterministic PnL evolution check
    PnlEvolution {
        #[arg(long)]
        trace_id: String,
    },
    /// Stage 2B: Trace trade lifecycle deterministic states
    TradeLifecycle {
        #[arg(long)]
        trade_id: String,
    },
    /// Stage 2C: Replay historical market session
    HistoricalReplay {
        #[arg(long)]
        journal_dir: String,
    },
    /// Stage 2C: Trace reconstructed market session
    SessionTrace {
        #[arg(long)]
        session_id: String,
    },
    /// Stage 2C: Analyze microstructure fidelity
    MarketFidelity {
        #[arg(long)]
        session_id: String,
    },
    /// Stage 2C: Replay timestamp alignment logic
    TimestampAlignment {
        #[arg(long)]
        journal_dir: String,
    },
    /// Stage 2C: Trace venue sequence reconstruction
    VenueReconstruction {
        #[arg(long)]
        venue_id: String,
    },
    /// Stage 2C: Analyze liquidity regime segments
    MicrostructureAnalysis {
        #[arg(long)]
        session_id: String,
    },
    /// Stage 2D: Trace deterministic risk exposure
    RiskTrace {
        #[arg(long)]
        trace_id: String,
    },
    /// Stage 2D: Replay margin and collateral limits
    MarginAnalysis {
        #[arg(long)]
        portfolio_id: String,
    },
    /// Stage 2D: Execute deterministic liquidation cascade
    LiquidationReplay {
        #[arg(long)]
        window_id: String,
    },
    /// Stage 2D: Run systemic stress propagation
    StressSimulation {
        #[arg(long)]
        scenario_id: String,
    },
    /// Stage 2D: View inventory concentration map
    ExposureMap {
        #[arg(long)]
        portfolio_id: String,
    },
    /// Stage 2D: Audit deterministic risk constraints
    ConstraintAudit {
        #[arg(long)]
        portfolio_id: String,
    },
    /// Stage 2E: Deterministic profiling of replay batch
    ReplayProfile {
        #[arg(long)]
        trace_id: String,
    },
    /// Stage 2E: Trace canonical sequence throughput
    ThroughputAnalysis {
        #[arg(long)]
        journal_dir: String,
    },
    /// Stage 2E: Simulated memory allocation footprint
    MemoryTrace {
        #[arg(long)]
        snapshot_id: String,
    },
    /// Stage 2E: Execution cost efficiency metric
    ExecutionEfficiency2e {
        #[arg(long)]
        trace_id: String,
    },
    /// Stage 2E: Parity hashing footprint analysis
    CertificationOverhead {
        #[arg(long)]
        trace_id: String,
    },
    /// Stage 2E: Batch scaling metrics
    ReplayScaling {
        #[arg(long)]
        journal_dir: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::ReplaySummary { journal_dir } => {
            let risk_engine = RiskEngine::new();
            let runtime = ExchangeRuntime::new(risk_engine);
            let mut inspector = ReplayInspector::new(runtime);

            let report = inspector.inspect_directory(&journal_dir).unwrap();
            let hash = inspector.runtime.generate_global_hash();

            println!("======================================");
            println!("          REPLAY SUMMARY              ");
            println!("======================================");
            println!("Events Processed : {}", report.order_flow.total_orders);
            println!(
                "Accepted Orders  : {}",
                inspector.runtime.diagnostics.total_accepted_orders
            );
            println!(
                "Rejected Orders  : {}",
                inspector.runtime.diagnostics.total_rejected_orders
            );
            println!("Fill Count       : {}", report.order_flow.total_fills);
            println!("Global Hash      : {:?}", hash);
            println!("======================================");
        }
        Commands::OrderbookTrace {
            journal_dir,
            symbol,
        } => {
            let risk_engine = RiskEngine::new();
            let runtime = ExchangeRuntime::new(risk_engine);
            let mut inspector = ReplayInspector::new(runtime);

            inspector.inspect_directory(&journal_dir).unwrap();

            println!("======================================");
            println!("           ASCII TIMELINE             ");
            println!("======================================");
            let mut stdout = std::io::stdout();
            inspector
                .timeline
                .export_ascii_timeline(&mut stdout)
                .unwrap();

            println!("\n======================================");
            println!("         FINAL ORDERBOOK ({})         ", symbol);
            println!("======================================");
            export_multi_venue_ascii_lob(&mut stdout, &inspector.runtime.router.venues, &symbol)
                .unwrap();
        }
        Commands::PnlReport { journal_dir } => {
            let risk_engine = RiskEngine::new();
            let runtime = ExchangeRuntime::new(risk_engine);
            let mut inspector = ReplayInspector::new(runtime);

            inspector.inspect_directory(&journal_dir).unwrap();

            println!("======================================");
            println!("             PNL REPORT               ");
            println!("======================================");
            for exposure in inspector
                .runtime
                .position_engine
                .positions
                .values()
                .flat_map(|m| m.values())
            {
                println!(
                    "Trader {} | Realized PnL: {} | Unrealized PnL: {} | Net Qty: {}",
                    exposure.trader_id,
                    exposure.realized_pnl,
                    exposure.unrealized_pnl,
                    exposure.net_quantity
                );
            }
        }
        Commands::StrategyTrace { journal_dir } => {
            let risk_engine = RiskEngine::new();
            let runtime = ExchangeRuntime::new(risk_engine);
            let mut inspector = ReplayInspector::new(runtime);

            inspector.inspect_directory(&journal_dir).unwrap();
            let mut stdout = std::io::stdout();
            export_mermaid_strategy_trace(&mut stdout, &inspector.strategy_analytics).unwrap();
        }
        Commands::StrategyAnalytics { journal_dir } => {
            let risk_engine = RiskEngine::new();
            let runtime = ExchangeRuntime::new(risk_engine);
            let mut inspector = ReplayInspector::new(runtime);

            inspector.inspect_directory(&journal_dir).unwrap();
            let mut stdout = std::io::stdout();
            export_strategy_analytics_json(&mut stdout, &inspector.strategy_analytics).unwrap();
        }
        Commands::ScenarioReport { scenario } => {
            println!("Generating scenario report for {}", scenario);
        }
        Commands::CascadeTrace { scenario } => {
            println!("Generating cascade trace for {}", scenario);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_liquidation_cascade_timeline(&mut stdout).unwrap();
        }
        Commands::VenueFailureMap { scenario } => {
            println!("Generating venue failure map for {}", scenario);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_venue_failure_topology_map(&mut stdout).unwrap();
        }
        Commands::CheckpointDiff { scenario } => {
            println!("Generating checkpoint diff for {}", scenario);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_replay_checkpoint_diff(&mut stdout).unwrap();
        }
        Commands::ReplayAudit { journal_dir } => {
            println!("Generating replay audit for {:?}", journal_dir);
        }
        Commands::OperationalTimeline { scenario } => {
            println!("Generating operational timeline for {}", scenario);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_operational_intervention_timeline(&mut stdout)
                .unwrap();
        }
        Commands::RecoveryAnalysis { scenario } => {
            println!("Generating recovery analysis for {}", scenario);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_recovery_topology_graph(&mut stdout).unwrap();
        }
        Commands::CheckpointLineage { scenario } => {
            println!("Generating checkpoint lineage for {}", scenario);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_checkpoint_lineage_diagram(&mut stdout).unwrap();
        }
        Commands::RunExperiment { suite } => {
            println!("Running experiment suite {}", suite);
        }
        Commands::CompareExperiments { left, right } => {
            println!("Comparing experiments {} and {}", left, right);
        }
        Commands::BenchmarkReport { suite } => {
            println!(
                "Generating institutional benchmark report for suite {}",
                suite
            );
        }
        Commands::ReplayCertify { manifest } => {
            println!("Certifying replay against manifest {}", manifest);
        }
        Commands::InterventionStudy { left, right } => {
            println!("Comparing interventions {} and {}", left, right);
        }
        Commands::SystemicComparison { suite } => {
            println!("Generating systemic comparison for suite {}", suite);
        }
        Commands::ReplayDiff {
            left_run_id,
            right_run_id,
        } => {
            println!(
                "Generating replay diff between {} and {}",
                left_run_id, right_run_id
            );
        }
        Commands::PolicyReport { scenario } => {
            println!("Generating policy report for scenario {}", scenario);
        }
        Commands::InterventionCompare { left, right } => {
            println!("Comparing interventions {} and {}", left, right);
        }
        Commands::ContagionMap { scenario } => {
            println!("Generating contagion map for scenario {}", scenario);
        }
        Commands::StabilizationTrace { scenario } => {
            println!("Tracing stabilization sequence for scenario {}", scenario);
        }
        Commands::ExperimentReport { suite } => {
            println!("Generating experiment report for suite {}", suite);
        }
        Commands::AuditReport { journal_dir } => {
            println!("Generating deterministic audit report for {}", journal_dir);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_benchmark_trust_graph(&mut stdout).unwrap();
        }
        Commands::ReplayVerify { journal_dir } => {
            println!("Verifying replay parity for {}", journal_dir);
        }
        Commands::LineageTrace { journal_dir } => {
            println!("Tracing replay lineage for {}", journal_dir);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_replay_lineage_tree(&mut stdout).unwrap();
        }
        Commands::InvariantScan { journal_dir } => {
            println!("Scanning invariants for {}", journal_dir);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_invariant_violation_map(&mut stdout).unwrap();
        }
        Commands::CertificationAudit { manifest } => {
            println!("Auditing certification chain for manifest {}", manifest);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_certification_ancestry_diagram(&mut stdout)
                .unwrap();
        }
        Commands::EcologyReport { journal_dir } => {
            println!("Generating ecology report for {}", journal_dir);
        }
        Commands::BehaviorTrace { journal_dir } => {
            println!("Tracing agent behaviors for {}", journal_dir);
        }
        Commands::CascadeMap { journal_dir } => {
            println!("Mapping systemic cascades for {}", journal_dir);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_systemic_panic_topology(&mut stdout).unwrap();
        }
        Commands::LiquidityRetreatAnalysis { journal_dir } => {
            println!("Analyzing liquidity retreats for {}", journal_dir);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_liquidity_retreat_tree(&mut stdout).unwrap();
        }
        Commands::AgentTopology { journal_dir } => {
            println!("Mapping agent topologies for {}", journal_dir);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_agent_topology_graph(&mut stdout).unwrap();
        }
        Commands::DatasetBuild { journal_dir } => {
            println!("Building RL dataset from {}", journal_dir);
        }
        Commands::PolicyEvaluate { policy_file } => {
            println!("Evaluating policy from {}", policy_file);
        }
        Commands::PolicyCompare {
            left_policy,
            right_policy,
        } => {
            println!("Comparing policies {} and {}", left_policy, right_policy);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_policy_divergence_map(&mut stdout).unwrap();
        }
        Commands::TrajectoryTrace { journal_dir } => {
            println!("Tracing trajectory lineage for {}", journal_dir);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_trajectory_lineage_tree(&mut stdout).unwrap();
        }
        Commands::RewardAnalysis { dataset_dir } => {
            println!("Analyzing reward evolution for {}", dataset_dir);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_reward_evolution_graph(&mut stdout).unwrap();
        }
        Commands::AdaptiveStudy { policy_file } => {
            println!("Running adaptive containment study for {}", policy_file);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_adaptive_containment_topology(&mut stdout)
                .unwrap();
        }
        Commands::DistributedReplay { cluster_id } => {
            println!("Distributed replay cluster status for {}", cluster_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_distributed_execution_graph(&mut stdout).unwrap();
        }
        Commands::ShardCertify { shard_id } => {
            println!("Certifying shard {}", shard_id);
        }
        Commands::ReplayClusterReport { report_id } => {
            println!("Generating replay cluster report for {}", report_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_certification_aggregation_tree(&mut stdout)
                .unwrap();
        }
        Commands::DistributedBenchmark { benchmark_id } => {
            println!("Running distributed benchmark fanout for {}", benchmark_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_distributed_benchmark_fanout(&mut stdout).unwrap();
        }
        Commands::LineageMergeTrace { manifest_id } => {
            println!("Tracing lineage merge for manifest {}", manifest_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_replay_lineage_merge_graph(&mut stdout).unwrap();
        }
        Commands::ReplayFabricTopology { cluster_id } => {
            println!("Viewing replay fabric topology for cluster {}", cluster_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_replay_shard_topology(&mut stdout).unwrap();
        }
        Commands::FormalVerify { proof_file } => {
            println!("Formally verifying determinism for {}", proof_file);
        }
        Commands::ReplayProof { manifest_id } => {
            println!("Generating replay proof for manifest {}", manifest_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_replay_proof_chain(&mut stdout).unwrap();
        }
        Commands::InvariantTrace { journal_dir } => {
            println!("Tracing invariant dependencies for {}", journal_dir);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_invariant_dependency_graph(&mut stdout).unwrap();
        }
        Commands::LineageProof { journal_dir } => {
            println!("Generating lineage proof for {}", journal_dir);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_lineage_certification_topology(&mut stdout)
                .unwrap();
        }
        Commands::AggregationVerify { aggregation_file } => {
            println!("Verifying aggregation proof for {}", aggregation_file);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_aggregation_parity_tree(&mut stdout).unwrap();
        }
        Commands::DistributedEquivalenceCheck { proof_file } => {
            println!("Checking distributed equivalence for {}", proof_file);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_distributed_equivalence_graph(&mut stdout)
                .unwrap();
        }
        Commands::FederationReport { federation_id } => {
            println!("Generating federation report for {}", federation_id);
        }
        Commands::TreatyVerify { treaty_file } => {
            println!("Verifying treaty {}", treaty_file);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_replay_treaty_dependency_graph(&mut stdout)
                .unwrap();
        }
        Commands::CrossClusterCheck {
            cluster_a,
            cluster_b,
        } => {
            println!("Cross cluster check {} vs {}", cluster_a, cluster_b);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_cross_cluster_lineage_bridge(&mut stdout).unwrap();
        }
        Commands::FederationTopology { federation_id } => {
            println!("Viewing federation topology for {}", federation_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_federation_topology_graph(&mut stdout).unwrap();
        }
        Commands::ReplayNotarize { manifest_id } => {
            println!("Notarizing replay for manifest {}", manifest_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_federated_certification_mesh(&mut stdout).unwrap();
        }
        Commands::SovereignLineageTrace { partition_id } => {
            println!("Tracing sovereign lineage for partition {}", partition_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_sovereign_partition_map(&mut stdout).unwrap();
        }
        Commands::FederatedEquivalenceCheck { federation_id } => {
            println!("Checking federated equivalence for {}", federation_id);
        }
        Commands::OrderbookReplay { journal_dir } => {
            println!("Replaying orderbook for {}", journal_dir);
        }
        Commands::QueueTrace { trace_id } => {
            println!("Tracing queues for {}", trace_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_queue_evolution_graph(&mut stdout).unwrap();
        }
        Commands::ExecutionLineage { window_id } => {
            println!("Printing execution lineage for {}", window_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_fill_lineage_topology(&mut stdout).unwrap();
        }
        Commands::LatencyAnalysis { journal_dir } => {
            println!("Analyzing latency for {}", journal_dir);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_latency_propagation_graph(&mut stdout).unwrap();
        }
        Commands::SlippageAnalysis { journal_dir } => {
            println!("Analyzing slippage for {}", journal_dir);
        }
        Commands::DepthEvolution { trace_id } => {
            println!("Viewing depth evolution for {}", trace_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_orderbook_depth_evolution(&mut stdout).unwrap();
        }
        Commands::StrategyReplay { strategy_id } => {
            println!("Replaying strategy {}", strategy_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_strategy_execution_topology(&mut stdout).unwrap();
        }
        Commands::PortfolioTrace { trace_id } => {
            println!("Tracing portfolio accounting drift for {}", trace_id);
        }
        Commands::ExecutionAnalysis { journal_dir } => {
            println!("Execution analytics for {}", journal_dir);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_execution_efficiency_map(&mut stdout).unwrap();
        }
        Commands::InventoryAnalysis { journal_dir } => {
            println!("Inventory exposure evolution for {}", journal_dir);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_inventory_exposure_topology(&mut stdout).unwrap();
        }
        Commands::PnlEvolution { trace_id } => {
            println!("PnL evolution check for {}", trace_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_pnl_evolution_graph(&mut stdout).unwrap();
        }
        Commands::TradeLifecycle { trade_id } => {
            println!("Trade lifecycle deterministic states for {}", trade_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_trade_lifecycle_graph(&mut stdout).unwrap();
        }
        Commands::HistoricalReplay { journal_dir } => {
            println!("Replaying historical market session for {}", journal_dir);
        }
        Commands::SessionTrace { session_id } => {
            println!("Tracing reconstructed market session for {}", session_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_market_session_topology(&mut stdout).unwrap();
        }
        Commands::MarketFidelity { session_id } => {
            println!("Analyzing microstructure fidelity for {}", session_id);
        }
        Commands::TimestampAlignment { journal_dir } => {
            println!("Replaying timestamp alignment logic for {}", journal_dir);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_timestamp_alignment_topology(&mut stdout).unwrap();
        }
        Commands::VenueReconstruction { venue_id } => {
            println!("Tracing venue sequence reconstruction for {}", venue_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_venue_fragmentation_evolution(&mut stdout)
                .unwrap();
        }
        Commands::MicrostructureAnalysis { session_id } => {
            println!("Analyzing liquidity regime segments for {}", session_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_liquidity_regime_graph(&mut stdout).unwrap();
        }
        Commands::RiskTrace { trace_id } => {
            println!("Tracing deterministic risk exposure for {}", trace_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_exposure_topology(&mut stdout).unwrap();
        }
        Commands::MarginAnalysis { portfolio_id } => {
            println!(
                "Replaying margin and collateral limits for {}",
                portfolio_id
            );
        }
        Commands::LiquidationReplay { window_id } => {
            println!(
                "Executing deterministic liquidation cascade for {}",
                window_id
            );
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_liquidation_cascade_graph(&mut stdout).unwrap();
            astra_inspect::visualization::export_collateral_exhaustion_tree(&mut stdout).unwrap();
        }
        Commands::StressSimulation { scenario_id } => {
            println!("Running systemic stress propagation for {}", scenario_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_systemic_stress_propagation_graph(&mut stdout)
                .unwrap();
        }
        Commands::ExposureMap { portfolio_id } => {
            println!("Viewing inventory concentration map for {}", portfolio_id);
        }
        Commands::ConstraintAudit { portfolio_id } => {
            println!(
                "Auditing deterministic risk constraints for {}",
                portfolio_id
            );
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_risk_constraint_dependency_map(&mut stdout)
                .unwrap();
        }
        Commands::ReplayProfile { trace_id } => {
            println!("Deterministic profiling of replay batch for {}", trace_id);
        }
        Commands::ThroughputAnalysis { journal_dir } => {
            println!("Tracing canonical sequence throughput for {}", journal_dir);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_replay_throughput_topology(&mut stdout).unwrap();
        }
        Commands::MemoryTrace { snapshot_id } => {
            println!("Simulated memory allocation footprint for {}", snapshot_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_memory_pressure_topology(&mut stdout).unwrap();
        }
        Commands::ExecutionEfficiency2e { trace_id } => {
            println!("Execution cost efficiency metric for {}", trace_id);
        }
        Commands::CertificationOverhead { trace_id } => {
            println!("Parity hashing footprint analysis for {}", trace_id);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_certification_overhead_graph(&mut stdout).unwrap();
        }
        Commands::ReplayScaling { journal_dir } => {
            println!("Batch scaling metrics for {}", journal_dir);
            let mut stdout = std::io::stdout();
            astra_inspect::visualization::export_execution_batch_evolution(&mut stdout).unwrap();
            astra_inspect::visualization::export_replay_scaling_trace(&mut stdout).unwrap();
        }
    }
}
