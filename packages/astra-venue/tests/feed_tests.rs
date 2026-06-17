use astra_venue::binance::RawTrade;
use astra_venue::normalizer::TradeNormalizer;
use astra_core::hashing::DeterministicState;
use astra_core::events::EventType;

#[test]
fn test_price_normalization() {
    let trade = RawTrade {
        symbol: "BTCUSDT".to_string(),
        price_str: "45000.50".to_string(),
        quantity_str: "1.0".to_string(),
        timestamp_ms: 1000,
    };
    let event = TradeNormalizer::normalize(&trade, 1);
    
    // Convert JSON bytes back to struct to verify price
    let payload: serde_json::Value = serde_json::from_slice(&event.payload).unwrap();
    assert_eq!(payload["price"].as_u64().unwrap(), 4_500_050_000_000);
}

#[test]
fn test_raw_trade_to_astra_event() {
    let trade = RawTrade {
        symbol: "ETHUSDT".to_string(),
        price_str: "3000.00".to_string(),
        quantity_str: "0.5".to_string(),
        timestamp_ms: 1680000000000,
    };
    
    let event = TradeNormalizer::normalize(&trade, 42);
    
    assert_eq!(event.event_type, EventType::MarketTick);
    assert_eq!(event.timestamp_ns, 1680000000000 * 1_000_000);
    assert_eq!(event.sequence_id, 42);
}

#[test]
fn test_normalizer_determinism() {
    let trade = RawTrade {
        symbol: "SOLUSDT".to_string(),
        price_str: "100.1234".to_string(),
        quantity_str: "10.0".to_string(),
        timestamp_ms: 99999999,
    };
    
    let event1 = TradeNormalizer::normalize(&trade, 1);
    let event2 = TradeNormalizer::normalize(&trade, 1);
    
    assert_eq!(event1.state_hash(), event2.state_hash());
}
