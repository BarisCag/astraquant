use astra_core::events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata};
use astra_exchange::runtime::ExchangeRuntime;
use astra_risk::engine::RiskEngine;
use astra_scenarios::library::recovery_drill::RecoveryDrillScenario;
use astra_scenarios::orchestrator::ScenarioOrchestrator;
use astra_scenarios::scenario::ScenarioDefinition;

#[test]
fn test_recovery_drill_execution() {
    let scenario = RecoveryDrillScenario {
        seed: 42,
        venue_failure_sequence: 5,
        intervention_sequence: 15,
        target_venue: 1,
    };

    let mut orch = ScenarioOrchestrator::new(
        ExchangeRuntime::new(RiskEngine::new()),
        "recovery_drill".to_string(),
        42,
    );

    for i in 1..=20 {
        let base = AstraEvent {
            timestamp_ns: i * 1_000_000,
            sequence_id: i,
            event_type: EventType::MarketTick,
            payload: vec![],
            payload_metadata: PayloadMetadata::new(PayloadEncoding::RawBytes, 1),
        };
        orch.step(base, Some(&scenario));
    }

    // Verify venue 1 is Active (resumed at sequence 15)
    let venue = orch
        .exchange
        .router
        .venues
        .get(&astra_router::venue::VenueId(1))
        .unwrap();
    assert_eq!(venue.status, astra_router::venue::VenueStatus::Active);
}
