use crate::config::Config;
use crate::rbac::Role;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct ApiKey {
    pub key_hash: [u8; 32],
    pub user_id: String,
    pub role: Role,
}

#[derive(Clone)]
pub struct AuthManager {
    api_keys: HashMap<[u8; 32], ApiKey>,
    jwt_secret: [u8; 32],
    jwt_expiry_hours: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

impl AuthManager {
    pub fn from_config(config: &Config) -> Self {
        let mut api_keys = HashMap::new();
        for (hash_hex, role_str) in &config.api_keys {
            if let Ok(hash_bytes) = hex::decode(hash_hex) {
                if hash_bytes.len() == 32 {
                    let mut key_hash = [0u8; 32];
                    key_hash.copy_from_slice(&hash_bytes);
                    if let Some(role) = Role::from_str(role_str) {
                        api_keys.insert(
                            key_hash,
                            ApiKey {
                                key_hash,
                                user_id: format!("user_from_key_{}", &hash_hex[0..8]),
                                role,
                            },
                        );
                    }
                }
            }
        }

        let mut jwt_secret = [0u8; 32];
        let secret_bytes = config.auth.jwt_secret.as_bytes();
        let len = std::cmp::min(32, secret_bytes.len());
        jwt_secret[..len].copy_from_slice(&secret_bytes[..len]);

        Self {
            api_keys,
            jwt_secret,
            jwt_expiry_hours: config.auth.jwt_expiry_hours,
        }
    }

    pub fn validate_api_key(&self, raw_key: &str) -> Option<&ApiKey> {
        let hash = *blake3::hash(raw_key.as_bytes()).as_bytes();
        self.api_keys.get(&hash)
    }

    pub fn issue_jwt(&self, user_id: &str, role: Role) -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
        let exp = now + self.jwt_expiry_hours * 3600;

        let claims = Claims {
            sub: user_id.to_string(),
            role: format!("{:?}", role),
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.jwt_secret),
        )
        .unwrap()
    }

    pub fn validate_jwt(&self, token: &str) -> Option<Claims> {
        let mut validation = Validation::default();
        validation.leeway = 60;
        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(&self.jwt_secret),
            &validation,
        ) {
            Ok(token_data) => Some(token_data.claims),
            Err(_) => None,
        }
    }
}
