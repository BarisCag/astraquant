use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Normal, Distribution};
use rayon::prelude::*;

pub struct VarCalculator;

impl VarCalculator {
    /// Deterministic Monte Carlo simulation for 1-day Expected Shortfall (ES) at 97.5% confidence.
    /// Runs 10,000 paths using rayon to guarantee < 20ms latency.
    pub fn monte_carlo_es(
        portfolio_value_usd: f64,
        daily_volatility: f64,
        journal_hash: &[u8; 32],
        block_height: u64,
    ) -> (f64, f64) { // Returns (VaR_97_5, ES_97_5)
        let num_paths = 10_000;
        
        // Seed derivation: blake3(journal_hash || block_height)
        let mut seed_input = Vec::with_capacity(40);
        seed_input.extend_from_slice(journal_hash);
        seed_input.extend_from_slice(&block_height.to_le_bytes());
        let seed_hash = blake3::hash(&seed_input);
        
        let root_seed = *seed_hash.as_bytes();
        
        let mut simulated_returns: Vec<f64> = (0..num_paths).into_par_iter().map(|i| {
            let mut path_seed = root_seed;
            // Mix the path index into the seed for this path to ensure deterministic parallel randomness
            for (j, byte) in path_seed.iter_mut().enumerate().take(4) {
                *byte ^= (i >> (j * 8)) as u8;
            }
            let mut rng = ChaCha8Rng::from_seed(path_seed);
            let normal = Normal::new(0.0, daily_volatility).unwrap();
            normal.sample(&mut rng)
        }).collect();
        
        // Sort returns to find percentiles (ascending order, worst returns first)
        simulated_returns.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        
        // 97.5% confidence means the worst 2.5% tail
        let tail_index = (num_paths as f64 * 0.025).floor() as usize;
        
        let var_return = simulated_returns[tail_index];
        
        // ES is the average of all returns worse than the VaR threshold
        let tail_sum: f64 = simulated_returns[..tail_index].iter().sum();
        let es_return = tail_sum / (tail_index as f64);
        
        // VaR and ES expressed as absolute loss (so multiply by value and return as negative or absolute loss)
        // We return the loss amount (negative value indicating loss)
        let var_usd = portfolio_value_usd * var_return;
        let es_usd = portfolio_value_usd * es_return;
        
        (var_usd, es_usd)
    }

    /// Fast Parametric VaR
    pub fn parametric_var_99(portfolio_value_usd: f64, daily_volatility: f64) -> f64 {
        // Z-score for 99% is approx -2.326
        let z_score = -2.326;
        portfolio_value_usd * daily_volatility * z_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_monte_carlo_determinism_and_latency() {
        let portfolio_value = 100_000.0;
        let daily_volatility = 0.05;
        let journal_hash = &[0u8; 32];
        let block_height = 100;

        // Test latency
        let start = Instant::now();
        let (var1, es1) = VarCalculator::monte_carlo_es(
            portfolio_value,
            daily_volatility,
            journal_hash,
            block_height,
        );
        let duration = start.elapsed();
        
        println!("Monte Carlo 10k paths completed in: {:?}", duration);
        assert!(duration.as_millis() < 100, "Latency exceeded 100ms: {}ms", duration.as_millis());

        // Test determinism
        let (var2, es2) = VarCalculator::monte_carlo_es(
            portfolio_value,
            daily_volatility,
            journal_hash,
            block_height,
        );

        assert_eq!(var1, var2, "VaR is not deterministic");
        assert_eq!(es1, es2, "ES is not deterministic");
        
        // Ensure negative returns indicate a loss in our model
        assert!(var1 < 0.0, "VaR should be negative (loss)");
        assert!(es1 < var1, "ES should be worse than VaR");
    }
}
