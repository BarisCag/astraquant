use astra_api::auth::AuthManager;
use astra_api::config::Config;
use astra_api::rbac::Role;
use std::time::{SystemTime, UNIX_EPOCH};

fn test_config() -> Config {
    Config::load("config/default.toml")
}

#[test]
fn test_valid_api_key() {
    let mut config = test_config();
    let hash_hex = hex::encode(blake3::hash(b"trader_key").as_bytes());
    config.api_keys.insert(hash_hex, "Trader".to_string());
    
    let auth = AuthManager::from_config(&config);
    
    let key = auth.validate_api_key("trader_key");
    assert!(key.is_some());
    assert_eq!(key.unwrap().role, Role::Trader);
}

#[test]
fn test_invalid_api_key() {
    let config = test_config();
    let auth = AuthManager::from_config(&config);
    
    let key = auth.validate_api_key("invalid_key");
    assert!(key.is_none());
}

#[test]
fn test_valid_jwt() {
    let config = test_config();
    let auth = AuthManager::from_config(&config);
    
    let jwt = auth.issue_jwt("user_123", Role::Trader);
    let claims = auth.validate_jwt(&jwt);
    
    assert!(claims.is_some());
    let claims = claims.unwrap();
    assert_eq!(claims.sub, "user_123");
    assert_eq!(claims.role, "Trader");
}

#[test]
fn test_expired_jwt() {
    let config = test_config();
    let auth = AuthManager::from_config(&config);
    
    // We simulate expiration by setting expiry to 0 hours and issuing a token
    let _jwt = auth.issue_jwt("user_123", Role::Trader);
    
    // Actually, to simulate expiry, we can just let it expire or manually construct an expired token.
    // The standard test for jwt expiry would manually encode with exp < now.
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let expired_claims = astra_api::auth::Claims {
        sub: "user".to_string(),
        role: "Trader".to_string(),
        exp: now - 3600, // 1 hour ago
    };
    
    let expired_jwt = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &expired_claims,
        &jsonwebtoken::EncodingKey::from_secret(b"astraquant-dev-secret-change-in-prod"),
    ).unwrap();
    
    let result = auth.validate_jwt(&expired_jwt);
    assert!(result.is_none());
}
