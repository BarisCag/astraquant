use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationalCommand {
    PauseVenue {
        venue_id: u8,
    },
    ResumeVenue {
        venue_id: u8,
    },
    FreezeSettlementEngine,
    InjectRecoveryLiquidity {
        symbol: String,
        size: u64,
        price: u64,
    },
    EmergencyMarginIncrease {
        multiplier_ppm: u64,
    },
    RouteOverride {
        original_venue: u8,
        target_venue: u8,
    },
    ForcedLiquidationHalt,
    ScenarioInterruption,
    ReplayCheckpointRestore {
        sequence: u64,
        integrity_hash: [u8; 32],
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationalAction {
    pub operator_id: String,
    pub command: OperationalCommand,
    pub sequence_applied: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationalCheckpoint {
    pub sequence: u64,
    pub state_hash: [u8; 32],
    pub metadata: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayCertification {
    pub original_hash: [u8; 32],
    pub certified_hash: [u8; 32],
    pub total_interventions: u64,
    pub certified_by: String,
}
