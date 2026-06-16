use astra_core::events::{EventType, PayloadEncoding, PayloadMetadata};
use astra_core::journal::EventJournal;
use astra_core::orderbook::{LimitOrderPlacedPayload, OrderSide};
use astra_core::serialization::serialize_canonical;
use astra_core::types::{Price, Quantity};
use astra_exchange::runtime::ExchangeRuntime;
use astra_inspect::inspector::ReplayInspector;
use astra_risk::engine::RiskEngine;
use astra_risk::types::TraderRiskProfile;
use std::fs;
use std::path::PathBuf;

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }
    fn next_range(&mut self, min: u64, max: u64) -> u64 {
        let range = max - min + 1;
        min + (self.next() % range)
    }
}

fn synthesize_journal(dir: PathBuf, seed: u64, num_events: usize, reject_heavy: bool) -> PathBuf {
    fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("astra_19700101_00.astra_jl");
    let mut journal = EventJournal::create(&file_path, 0).unwrap();

    let mut lcg = Lcg::new(seed);

    for sequence_id in 1..=num_events as u64 {
        let is_bid = (lcg.next() >> 16).is_multiple_of(2);
        let side = if is_bid {
            OrderSide::Bid
        } else {
            OrderSide::Ask
        };

        let price = lcg.next_range(49000, 51000);
        let quantity = if reject_heavy && lcg.next().is_multiple_of(5) {
            lcg.next_range(1_000_000, 2_000_000) // Huge order to trigger risk rejection
        } else {
            lcg.next_range(1, 100)
        };
        let trader_id = lcg.next_range(1, 10);

        let payload = LimitOrderPlacedPayload {
            order_id: sequence_id,
            trader_id,
            symbol: "BTC/USD".to_string(),
            side,
            price: Price::new(price as i64),
            quantity: Quantity::new(quantity),
        };

        journal
            .commit(
                sequence_id * 1_000_000,
                EventType::LimitOrderPlaced,
                serialize_canonical(&payload).unwrap(),
                PayloadMetadata::new(PayloadEncoding::Bincode, 1),
            )
            .unwrap();
        
    }
    dir
}

fn setup_inspector() -> ReplayInspector {
    let mut risk_engine = RiskEngine::new();
    for i in 1..=10 {
        risk_engine.register_trader(TraderRiskProfile {
            trader_id: i,
            max_position_notional: 1_000_000_000_000, // Very high
            max_order_quantity: 1_000_000_000,
            max_drawdown: 1_000_000_000_000,
            max_order_velocity: 1000000,
        });
    }
    ReplayInspector::new(ExchangeRuntime::new(risk_engine))
}

#[test]
fn test_low_activity_fixture() {
    let _tempdir = tempfile::tempdir().unwrap();
    let dir = _tempdir.path().to_path_buf();
    synthesize_journal(dir.clone(), 42, 1000, false);

    let mut inspector = setup_inspector();
    let report = inspector.inspect_directory(&dir).unwrap();

    assert_eq!(report.order_flow.total_orders, 1000);
    // Everything is deterministic! Check that fills match.
    assert!(report.order_flow.total_fills > 0);
    assert_eq!(report.order_flow.total_rejections, 0);

    // Execution Quality Checks
    let eq = &inspector.execution_quality;
    assert!(eq.total_passive_quantity > 0);
    assert!(eq.spread_samples > 0);
    assert!(eq.average_effective_spread_x2() > 0);
}

#[test]
fn test_high_activity_fixture() {
    let _tempdir = tempfile::tempdir().unwrap();
    let dir = _tempdir.path().to_path_buf();
    synthesize_journal(dir.clone(), 43, 10000, false);

    let mut inspector = setup_inspector();
    let report = inspector.inspect_directory(&dir).unwrap();

    assert_eq!(report.order_flow.total_orders, 10000);
    assert!(report.order_flow.total_fills > 0);
    assert_eq!(report.order_flow.total_rejections, 0);
}

#[test]
fn test_rejection_heavy_fixture() {
    let _tempdir = tempfile::tempdir().unwrap();
    let dir = _tempdir.path().to_path_buf();
    synthesize_journal(dir.clone(), 44, 5000, true);

    let mut inspector = setup_inspector();
    let report = inspector.inspect_directory(&dir).unwrap();

    assert_eq!(report.order_flow.total_orders, 5000);
    assert!(report.order_flow.total_rejections > 0);
}

#[test]
fn test_deterministic_exports() {
    let _tempdir1 = tempfile::tempdir().unwrap();
    let _tempdir2 = tempfile::tempdir().unwrap();
    let dir1 = _tempdir1.path().to_path_buf();
    let dir2 = _tempdir2.path().to_path_buf();

    synthesize_journal(dir1.clone(), 123, 500, false);
    synthesize_journal(dir2.clone(), 123, 500, false);

    let mut inspector1 = setup_inspector();
    let report1 = inspector1.inspect_directory(&dir1).unwrap();
    let mut out1 = Vec::new();
    astra_inspect::visualization::export_csv(&mut out1, &inspector1.timeline).unwrap();

    let mut inspector2 = setup_inspector();
    let report2 = inspector2.inspect_directory(&dir2).unwrap();
    let mut out2 = Vec::new();
    astra_inspect::visualization::export_csv(&mut out2, &inspector2.timeline).unwrap();

    assert_eq!(report1.total_journal_bytes, report2.total_journal_bytes);
    assert_eq!(
        report1.order_flow.total_fills,
        report2.order_flow.total_fills
    );
    assert_eq!(out1, out2); // CSV bytes must be EXACTLY identical

    // Verify Execution Quality Determinism
    assert_eq!(
        inspector1.execution_quality.total_effective_spread_x2,
        inspector2.execution_quality.total_effective_spread_x2
    );
    assert_eq!(
        inspector1.execution_quality.total_queue_advancement,
        inspector2.execution_quality.total_queue_advancement
    );
}

#[test]
fn test_execution_quality_metrics() {
    let _tempdir = tempfile::tempdir().unwrap();
    let dir = _tempdir.path().to_path_buf();
    synthesize_journal(dir.clone(), 999, 2000, false);

    let mut inspector = setup_inspector();
    inspector.inspect_directory(&dir).unwrap();

    let eq = inspector.execution_quality;
    let passive_fill_ratio = eq.passive_fill_ratio_ppm();
    let cancel_fill_ratio = eq.cancel_to_fill_ratio_ppm();
    let queue_survival = eq.queue_survival_ratio_ppm();

    // The ratio should be deterministic and bounded
    assert!(passive_fill_ratio <= 1_000_000);
    assert!(cancel_fill_ratio <= 1_000_000);
    assert!(queue_survival <= 1_000_000);
}
