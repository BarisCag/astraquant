use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiry_hours: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DemoConfig {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u64,
    pub burst: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub demo: DemoConfig,
    pub rate_limit: RateLimitConfig,
    pub api_keys: HashMap<String, String>, // key_hash -> role
}

impl Config {
    pub fn load(path: &str) -> Self {
        let content = fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {}", path));
        toml::from_str(&content).unwrap_or_else(|_| panic!("Failed to parse {}", path))
    }
}
