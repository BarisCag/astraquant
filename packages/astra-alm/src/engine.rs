use crate::types::{ALMMismatchReport, Currency, RateSensitivity, TenorBucket};
use blake3::Hasher;
use std::collections::HashMap;

pub struct ALMEngine {
    assets: Vec<RateSensitivity>,
    liabilities: Vec<RateSensitivity>,
}

impl ALMEngine {
    pub fn new() -> Self {
        Self {
            assets: Vec::new(),
            liabilities: Vec::new(),
        }
    }

    pub fn add_asset(&mut self, asset: RateSensitivity) {
        self.assets.push(asset);
    }

    pub fn add_liability(&mut self, liability: RateSensitivity) {
        self.liabilities.push(liability);
    }

    pub fn compute_mismatch(&self) -> Vec<ALMMismatchReport> {
        let mut buckets: HashMap<(TenorBucket, Currency), (f64, i64, f64, i64)> = HashMap::new();
        
        // (asset_duration_sum, asset_notional, liability_duration_sum, liability_notional)

        for a in &self.assets {
            let key = (a.tenor, a.currency);
            let entry = buckets.entry(key).or_insert((0.0, 0, 0.0, 0));
            entry.0 += a.modified_duration * (a.notional as f64);
            entry.1 += a.notional;
        }

        for l in &self.liabilities {
            let key = (l.tenor, l.currency);
            let entry = buckets.entry(key).or_insert((0.0, 0, 0.0, 0));
            entry.2 += l.modified_duration * (l.notional as f64);
            entry.3 += l.notional;
        }

        let mut reports = Vec::new();

        for ((tenor, currency), (a_dur_sum, a_not, l_dur_sum, l_not)) in buckets {
            let asset_duration = if a_not > 0 { a_dur_sum / (a_not as f64) } else { 0.0 };
            let liability_duration = if l_not > 0 { l_dur_sum / (l_not as f64) } else { 0.0 };

            reports.push(ALMMismatchReport {
                tenor,
                currency,
                asset_duration,
                liability_duration,
                duration_gap: asset_duration - liability_duration,
                notional_gap: a_not - l_not,
            });
        }

        reports
    }

    pub fn state_hash(&self) -> [u8; 32] {
        let mut hasher = Hasher::new();
        
        // Sort or hash deterministically. Since we just need to hash the lists:
        for a in &self.assets {
            hasher.update(&(a.tenor as u8).to_le_bytes());
            hasher.update(&(a.currency as u8).to_le_bytes());
            hasher.update(&a.modified_duration.to_bits().to_le_bytes());
            hasher.update(&a.convexity.to_bits().to_le_bytes());
            hasher.update(&a.notional.to_le_bytes());
        }
        
        for l in &self.liabilities {
            hasher.update(&(l.tenor as u8).to_le_bytes());
            hasher.update(&(l.currency as u8).to_le_bytes());
            hasher.update(&l.modified_duration.to_bits().to_le_bytes());
            hasher.update(&l.convexity.to_bits().to_le_bytes());
            hasher.update(&l.notional.to_le_bytes());
        }

        hasher.finalize().into()
    }
}

impl Default for ALMEngine {
    fn default() -> Self {
        Self::new()
    }
}
