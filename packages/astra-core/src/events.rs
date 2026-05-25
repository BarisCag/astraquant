use crate::hashing::{hash_bytes, DeterministicState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PayloadEncoding {
    RawBytes,
    Bincode,
    Json,
    Protobuf,
    ArrowIPC,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PayloadMetadata {
    pub encoding: PayloadEncoding,
    pub schema_version: u16,
}

impl PayloadMetadata {
    pub fn new(encoding: PayloadEncoding, schema_version: u16) -> Self {
        Self {
            encoding,
            schema_version,
        }
    }
    pub fn raw() -> Self {
        Self::new(PayloadEncoding::RawBytes, 1)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventType {
    MarketTick = 1,
    LimitOrderPlaced = 2,
    RiskLimitBreached = 3,
    LimitOrderCancelled = 4,
    LimitOrderMatched = 5,
    PositionOpened = 6,
    PositionClosed = 7,
    RiskThresholdTriggered = 8,
    SystemRecovery = 9,
    OperatorAction = 10,
    TradeSettled = 11,
    StateSnapshot = 12,
    OrderSubmitted = 13,
    OrderFilled = 14,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AstraEvent {
    pub timestamp_ns: u64,
    pub sequence_id: u64,
    pub event_type: EventType,
    pub payload: Vec<u8>,
    pub payload_metadata: PayloadMetadata,
}

impl AstraEvent {
    pub fn new(
        timestamp_ns: u64,
        sequence_id: u64,
        event_type: EventType,
        payload: Vec<u8>,
        payload_metadata: PayloadMetadata,
    ) -> Self {
        Self {
            timestamp_ns,
            sequence_id,
            event_type,
            payload,
            payload_metadata,
        }
    }

    pub fn new_raw(
        timestamp_ns: u64,
        sequence_id: u64,
        event_type: EventType,
        payload: Vec<u8>,
    ) -> Self {
        Self::new(
            timestamp_ns,
            sequence_id,
            event_type,
            payload,
            PayloadMetadata::raw(),
        )
    }
}

impl DeterministicState for AstraEvent {
    fn state_hash(&self) -> [u8; 32] {
        let mut data = Vec::new();
        data.extend_from_slice(&self.timestamp_ns.to_le_bytes());
        data.extend_from_slice(&self.sequence_id.to_le_bytes());
        data.push(self.event_type as u8);
        data.extend_from_slice(&self.payload);
        hash_bytes(&data)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotMetadata {
    pub snapshot_id: String,
    pub state_hash: [u8; 32],
    pub last_sequence_id: u64,
}

impl SnapshotMetadata {
    pub fn from_hash(last_sequence_id: u64, state_hash: [u8; 32], snapshot_id: String) -> Self {
        Self {
            snapshot_id,
            state_hash,
            last_sequence_id,
        }
    }
}
