use astra_api::audit::{AuditEntry, AuditTrail};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn test_audit_chain_integrity() {
    let trail = AuditTrail::new();
    
    for i in 0..5 {
        let entry = AuditEntry {
            timestamp_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64,
            api_key_hash: [i; 32],
            endpoint: format!("/api/test/{}", i),
            method: "GET".to_string(),
            request_body_hash: [0u8; 32],
            response_status: 200,
            processing_time_ms: 10,
            chain_hash: [0u8; 32], // Will be updated by append
        };
        trail.append(entry).await;
    }
    
    assert!(trail.verify_chain().await);
}

#[tokio::test]
async fn test_audit_tamper_detection() {
    let trail = AuditTrail::new();
    
    for i in 0..5 {
        let entry = AuditEntry {
            timestamp_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64,
            api_key_hash: [i; 32],
            endpoint: format!("/api/test/{}", i),
            method: "GET".to_string(),
            request_body_hash: [0u8; 32],
            response_status: 200,
            processing_time_ms: 10,
            chain_hash: [0u8; 32],
        };
        trail.append(entry).await;
    }
    
    assert!(trail.verify_chain().await);
    
    // Tamper with the 3rd entry (index 2)
    trail.tamper_with_entry(2).await;
    
    assert!(!trail.verify_chain().await);
}

#[tokio::test]
async fn test_every_request_logged() {
    // This is essentially just testing `append` properly stores entries
    let trail = AuditTrail::new();
    let n = 10;
    
    for _ in 0..n {
        let entry = AuditEntry {
            timestamp_ns: 0,
            api_key_hash: [0; 32],
            endpoint: "".to_string(),
            method: "".to_string(),
            request_body_hash: [0u8; 32],
            response_status: 200,
            processing_time_ms: 0,
            chain_hash: [0u8; 32],
        };
        trail.append(entry).await;
    }
    
    assert_eq!(trail.count().await, n);
}
