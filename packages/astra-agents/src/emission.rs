use crate::agent::AgentIntent;
use astra_core::events::{AstraEvent, EventType, PayloadMetadata, PayloadEncoding};

pub struct IntentEmitter;

impl IntentEmitter {
    pub fn emit(intent: &AgentIntent, sequence_id: u64) -> AstraEvent {
        AstraEvent {
            event_type: EventType::AgentIntent,
            payload: bincode::serialize(intent).unwrap_or_default(),
            timestamp_ns: 0,
            sequence_id,
            payload_metadata: PayloadMetadata { 
                encoding: PayloadEncoding::Bincode, 
                schema_version: 1 
            },
        }
    }
}

pub struct BehaviorEventTranslator;

pub struct AgentExecutionBatch {
    pub sequence: u64,
    pub intents: Vec<AgentIntent>,
}

pub struct CanonicalIntentWindow;
