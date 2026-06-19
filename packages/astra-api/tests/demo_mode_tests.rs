use astra_api::demo::DemoMode;

#[test]
fn test_positions_sanitized() {
    let demo = DemoMode { enabled: true };
    assert_eq!(demo.sanitize_position(5_250_000), 5_000_000);
    assert_eq!(demo.sanitize_position(5_750_000), 6_000_000);
    
    let live = DemoMode { enabled: false };
    assert_eq!(live.sanitize_position(5_250_000), 5_250_000);
}

#[test]
fn test_pnl_sanitized() {
    let demo = DemoMode { enabled: true };
    assert_eq!(demo.sanitize_pnl(125_450), 130_000);
    assert_eq!(demo.sanitize_pnl(124_450), 120_000);
}

#[test]
fn test_instruments_anonymized() {
    let demo = DemoMode { enabled: true };
    let sanitized = demo.sanitize_instrument("BTC-USD");
    assert!(sanitized.starts_with("INST_"));
    assert_eq!(sanitized.len(), 5 + 4); // INST_ + 4 hex chars
    
    let live = DemoMode { enabled: false };
    assert_eq!(live.sanitize_instrument("BTC-USD"), "BTC-USD");
}
