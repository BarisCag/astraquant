use std::env;

#[derive(Clone, Debug)]
pub struct DemoMode {
    pub enabled: bool,
}

impl DemoMode {
    pub fn from_env() -> Self {
        let enabled = env::var("DEMO_MODE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);
            
        Self { enabled }
    }

    pub fn sanitize_position(&self, notional: i64) -> i64 {
        if self.enabled {
            // Round to nearest 1,000,000
            ((notional as f64 / 1_000_000.0).round() * 1_000_000.0) as i64
        } else {
            notional
        }
    }

    pub fn sanitize_pnl(&self, pnl: i64) -> i64 {
        if self.enabled {
            // Round to nearest 10,000
            ((pnl as f64 / 10_000.0).round() * 10_000.0) as i64
        } else {
            pnl
        }
    }

    pub fn sanitize_instrument(&self, id: &str) -> String {
        if self.enabled {
            format!("INST_{}", hex::encode(blake3::hash(id.as_bytes()).as_bytes())[0..4].to_uppercase())
        } else {
            id.to_string()
        }
    }

    pub fn is_demo(&self) -> bool {
        self.enabled
    }
}
