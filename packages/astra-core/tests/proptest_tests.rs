use astra_core::events::{AstraEvent, EventType, PayloadEncoding, PayloadMetadata};
use astra_core::serialization::{deserialize_event, serialize_event};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_event_serialization_fuzz(
        sequence_id in any::<u64>(),
        timestamp_ns in any::<u64>(),
        payload in prop::collection::vec(any::<u8>(), 0..100)
    ) {
        let event = AstraEvent::new(
            timestamp_ns,
            sequence_id,
            EventType::MarketTick,
            payload.clone(),
            PayloadMetadata::new(PayloadEncoding::RawBytes, 1),
        );

        let bytes = serialize_event(&event).unwrap();
        let recovered = deserialize_event(&bytes).unwrap();

        assert_eq!(event.sequence_id, recovered.sequence_id);
        assert_eq!(event.timestamp_ns, recovered.timestamp_ns);
        assert_eq!(event.payload, recovered.payload);
    }
}
