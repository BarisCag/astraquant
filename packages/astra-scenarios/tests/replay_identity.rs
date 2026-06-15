use astra_exchange::runtime::ExchangeRuntime;
use astra_risk::engine::RiskEngine;
use astra_scenarios::orchestrator::ScenarioOrchestrator;
use astra_scenarios::library::flash_crash::FlashCrashScenario;
use astra_scenarios::scenario::{ScenarioDefinition, ScenarioSeverity};
use astra_core::events::{AstraEvent, EventType, PayloadMetadata, PayloadEncoding};

#[test]
fn test_scenario_replay_identity() {
    let severity = ScenarioSeverity {
        liquidity_drop_ppm: 50_000,
        latency_spike_multiplier: 1,
        collateral_haircut_ppm: 0,
        margin_requirement_multiplier: 1,
        probability_of_venue_failure_ppm: 0,
    };
    
    let scenario = FlashCrashScenario {
        seed: 42,
        severity: severity.clone(),
        activation_windows: vec![(10, 20)],
    };

    // Run 1
    let mut orch1 = ScenarioOrchestrator::new(ExchangeRuntime::new(RiskEngine::new()), "flash_crash".to_string(), 42);
    for i in 1..=30 {
        let base = AstraEvent {
            timestamp_ns: i * 1000,
            sequence_id: i,
            event_type: EventType::MarketTick,
            payload: vec![],
            payload_metadata: PayloadMetadata::new(PayloadEncoding::RawBytes, 1),
        };
        orch1.step(base, Some(&scenario));
    }
    let chk1 = orch1.create_checkpoint();

    // Run 2
    let mut orch2 = ScenarioOrchestrator::new(ExchangeRuntime::new(RiskEngine::new()), "flash_crash".to_string(), 42);
    for i in 1..=30 {
        let base = AstraEvent {
            timestamp_ns: i * 1000,
            sequence_id: i,
            event_type: EventType::MarketTick,
            payload: vec![],
            payload_metadata: PayloadMetadata::new(PayloadEncoding::RawBytes, 1),
        };
        orch2.step(base, Some(&scenario));
    }
    let chk2 = orch2.create_checkpoint();

    assert_eq!(chk1.integrity_hash, chk2.integrity_hash);
}
