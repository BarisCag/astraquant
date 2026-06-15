use astra_core::{
    deserialize_event, hash_bytes, hash_to_hex, serialize_event, verify_hash_equality, AstraEvent,
    DeterministicState, EventType, PayloadEncoding, PayloadMetadata, SnapshotMetadata,
};

#[test]
fn test_market_tick_roundtrip() {
    let event = AstraEvent::new(
        1609459200000000000u64,
        1,
        EventType::MarketTick,
        vec![0x01, 0xF4],
        PayloadMetadata::new(PayloadEncoding::RawBytes, 1),
    );
    let bytes = serialize_event(&event).unwrap();
    let recovered = deserialize_event(&bytes).unwrap();
    assert_eq!(event, recovered);
}

#[test]
fn test_order_submitted_roundtrip() {
    let event = AstraEvent::new(
        1609459205000000000u64,
        2,
        EventType::OrderSubmitted,
        vec![0x00, 0x00, 0x00, 0x01, 0x42, 0x75, 0x79, 0x00],
        PayloadMetadata::new(PayloadEncoding::Bincode, 1),
    );
    let bytes = serialize_event(&event).unwrap();
    let recovered = deserialize_event(&bytes).unwrap();
    assert_eq!(
        recovered.payload_metadata.encoding,
        PayloadEncoding::Bincode
    );
    assert_eq!(event, recovered);
}

#[test]
fn test_order_filled_roundtrip() {
    let event = AstraEvent::new_raw(
        1609459210000000000u64,
        3,
        EventType::OrderFilled,
        vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x00],
    );
    let bytes = serialize_event(&event).unwrap();
    assert_eq!(event, deserialize_event(&bytes).unwrap());
}

#[test]
fn test_risk_limit_breached_roundtrip() {
    let event = AstraEvent::new_raw(
        1609459215000000000u64,
        4,
        EventType::RiskLimitBreached,
        vec![0x4D, 0x61, 0x78, 0x20, 0x50, 0x4F, 0x53],
    );
    let bytes = serialize_event(&event).unwrap();
    assert_eq!(event, deserialize_event(&bytes).unwrap());
}

#[test]
fn test_state_snapshot_roundtrip() {
    let event = AstraEvent::new(
        1609459220000000000u64,
        5,
        EventType::StateSnapshot,
        vec![0x7B, 0x22, 0x73, 0x74, 0x61, 0x74, 0x65, 0x22, 0x7D],
        PayloadMetadata::new(PayloadEncoding::Json, 1),
    );
    let bytes = serialize_event(&event).unwrap();
    let recovered = deserialize_event(&bytes).unwrap();
    assert_eq!(recovered.payload_metadata.encoding, PayloadEncoding::Json);
    assert_eq!(event, recovered);
}

#[test]
fn test_sequence_ordering() {
    let e1 = AstraEvent::new_raw(1000000000, 1, EventType::MarketTick, vec![1]);
    let e2 = AstraEvent::new_raw(2000000000, 2, EventType::OrderSubmitted, vec![2]);
    let e3 = AstraEvent::new_raw(3000000000, 3, EventType::OrderFilled, vec![3]);
    assert!(e1.sequence_id < e2.sequence_id);
    assert!(e2.sequence_id < e3.sequence_id);
}

#[test]
fn test_multiple_events_deterministic() {
    let events: Vec<AstraEvent> = (1..=10)
        .map(|i| {
            AstraEvent::new_raw(
                1000000000 + i * 1000000,
                i,
                EventType::MarketTick,
                vec![i as u8],
            )
        })
        .collect();
    let serialized: Vec<Vec<u8>> = events.iter().map(|e| serialize_event(e).unwrap()).collect();
    let deserialized: Vec<AstraEvent> = serialized
        .iter()
        .map(|b| deserialize_event(b).unwrap())
        .collect();
    for (orig, rec) in events.iter().zip(deserialized.iter()) {
        assert_eq!(orig, rec);
    }
}

#[test]
fn test_nanosecond_precision() {
    let ts = 1234567890123456789u64;
    let event = AstraEvent::new_raw(ts, 1, EventType::MarketTick, vec![0xFF]);
    let bytes = serialize_event(&event).unwrap();
    assert_eq!(deserialize_event(&bytes).unwrap().timestamp_ns, ts);
}

// === Cryptographic Binary Stability (blake3) ===

#[test]
fn test_binary_stability_cryptographic() {
    let event = AstraEvent::new(
        1609459200000000000u64,
        42,
        EventType::OrderFilled,
        vec![0xDE, 0xAD, 0xBE, 0xEF],
        PayloadMetadata::new(PayloadEncoding::RawBytes, 1),
    );
    let b1 = serialize_event(&event).unwrap();
    let b2 = serialize_event(&event).unwrap();
    assert_eq!(b1, b2);
    assert_eq!(hash_bytes(&b1), hash_bytes(&b2));
}

#[test]
fn test_state_hash_determinism() {
    let event = AstraEvent::new_raw(
        1609459200000000000u64,
        1,
        EventType::MarketTick,
        vec![1, 2, 3],
    );
    assert_eq!(event.state_hash(), event.state_hash());
    assert_eq!(hash_to_hex(&event.state_hash()).len(), 64);
}

#[test]
fn test_state_hash_survives_roundtrip() {
    let original = AstraEvent::new_raw(
        9999999999999u64,
        100,
        EventType::RiskLimitBreached,
        vec![0xCA, 0xFE],
    );
    let original_hash = original.state_hash();
    let recovered = deserialize_event(&serialize_event(&original).unwrap()).unwrap();
    assert_eq!(original_hash, recovered.state_hash());
}

#[test]
fn test_different_payloads_different_hashes() {
    let a = AstraEvent::new_raw(1000000000, 1, EventType::MarketTick, vec![1]);
    let b = AstraEvent::new_raw(1000000000, 1, EventType::MarketTick, vec![2]);
    assert_ne!(a.state_hash(), b.state_hash());
}

// === Payload Metadata Tests ===

#[test]
fn test_payload_metadata_all_encodings_roundtrip() {
    let encs = [
        PayloadEncoding::RawBytes,
        PayloadEncoding::Json,
        PayloadEncoding::Bincode,
        PayloadEncoding::Protobuf,
        PayloadEncoding::ArrowIPC,
    ];
    for (i, enc) in encs.into_iter().enumerate() {
        let event = AstraEvent::new(
            1000000000,
            (i + 1) as u64,
            EventType::MarketTick,
            vec![42],
            PayloadMetadata::new(enc, (i + 1) as u16),
        );
        let recovered = deserialize_event(&serialize_event(&event).unwrap()).unwrap();
        assert_eq!(event.payload_metadata, recovered.payload_metadata);
    }
}

// === Snapshot Metadata Tests ===

#[test]
fn test_snapshot_metadata_integrity() {
    let state_hash = hash_bytes(b"canonical state");
    let meta = SnapshotMetadata::from_hash(42, state_hash, "risk-engine".to_string());
    assert_eq!(meta.last_sequence_id, 42);
    assert_eq!(meta.state_hash, state_hash);
    assert_eq!(meta.state_hash, hash_bytes(b"canonical state"));
}

#[test]
fn test_verify_hash_equality_utility() {
    let hash_a = [1u8; 32];
    let hash_b = [2u8; 32];
    assert!(verify_hash_equality(&hash_a, &hash_a));
    assert!(!verify_hash_equality(&hash_a, &hash_b));
}

#[test]
fn test_three_run_parity() {
    use astra_core::exchange::ExchangeRuntime;
    use astra_core::kernel::AstraKernel;
    use astra_core::replay::EventReducer;
    use astra_core::risk::create_default_risk_engine;
    use astra_core::runtime::StrategyRuntime;
    use astra_core::types::{Money, Quantity};

    let limits = create_default_risk_engine(Money::new(10_000_000), Quantity::new(1_000));
    
    let mut kernel1 = AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits.clone())));
    let mut kernel2 = AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits.clone())));
    let mut kernel3 = AstraKernel::new(StrategyRuntime::new(ExchangeRuntime::new(limits)));

    let events: Vec<AstraEvent> = (1..=10)
        .map(|i| {
            AstraEvent::new_raw(
                1000000000 + i * 1000000,
                i,
                EventType::MarketTick,
                vec![i as u8],
            )
        })
        .collect();

    for event in &events {
        kernel1.apply(event).unwrap();
        kernel2.apply(event).unwrap();
        kernel3.apply(event).unwrap();
    }

    let hash1 = kernel1.state_hash();
    let hash2 = kernel2.state_hash();
    let hash3 = kernel3.state_hash();

    assert_eq!(hash1, hash2, "Kernel 1 and 2 hashes diverge!");
    assert_eq!(hash2, hash3, "Kernel 2 and 3 hashes diverge!");
}
