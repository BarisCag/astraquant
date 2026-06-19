use astra_core::events::{AstraEvent, EventType, PayloadMetadata, PayloadEncoding};
use astra_paper::events::{FillEvent, PaperEvent};
use astra_paper::types::Side;
use astra_treasury::book::TreasuryBook;
use astra_treasury::event::TreasuryEvent;
use astra_treasury::types::Currency;
use rust_decimal::Decimal;

fn create_fill_event(qty: u64, price: u64, side: Side, ts: u64) -> AstraEvent {
    let fill = FillEvent {
        symbol: "BTCUSDT".to_string(),
        side,
        fill_price: price,
        fill_quantity: qty,
        timestamp_ns: ts,
        model_used: "Naive".to_string(),
        prng_nonce_at_fill: 1,
    };
    let payload = astra_core::serialization::serialize_canonical(&PaperEvent::Fill(fill)).unwrap();
    AstraEvent::new(ts, 1, EventType::OrderFilled, payload, PayloadMetadata::new(PayloadEncoding::Bincode, 1))
}

fn create_eod_event(ts: u64) -> AstraEvent {
    AstraEvent::new(ts, 2, EventType::AuditCheckpoint, vec![], PayloadMetadata::new(PayloadEncoding::RawBytes, 1))
}

#[test]
fn test_treasury_settlement_and_accrual() {
    let mut book = TreasuryBook::new(3); // 3 days runway threshold

    // Buy 10 BTC at 50,000 USD (Notional = 500,000 USD). Base balance = 100,000 USD
    // We will breach runway if this settles because 100k - 500k = -400k (Liquidity Warning)
    let fill = create_fill_event(10 * 100_000_000, 50_000 * 100_000_000, Side::Buy, 1000);
    
    let events = book.process_event(&fill);
    
    let pos = book.positions.get(&Currency::USD).unwrap();
    assert_eq!(pos.pending_outflows, Decimal::from(500_000));
    assert_eq!(pos.next_settlement_days, 2);
    
    // We expect 1 event: CashPositionUpdate
    assert_eq!(events.len(), 1);
    
    // Next day (EOD 1)
    let eod1 = create_eod_event(2000);
    let events1 = book.process_event(&eod1);
    
    let pos1 = book.positions.get(&Currency::USD).unwrap();
    assert_eq!(pos1.next_settlement_days, 1);
    assert_eq!(pos1.available_balance, Decimal::from(100_000)); // Still not settled
    
    // Day 2 (EOD 2) - Settlement Day
    let eod2 = create_eod_event(3000);
    let events2 = book.process_event(&eod2);
    
    let pos2 = book.positions.get(&Currency::USD).unwrap();
    assert_eq!(pos2.next_settlement_days, 0);
    assert_eq!(pos2.available_balance, Decimal::from(-400_000)); // Settled!
    assert_eq!(pos2.pending_outflows, Decimal::ZERO);
}
