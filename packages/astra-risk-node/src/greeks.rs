use statrs::distribution::{Normal, ContinuousCDF};

#[derive(Debug, Clone, Copy)]
pub struct GreeksProfile {
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
}

pub struct GreeksEngine;

impl GreeksEngine {
    /// Deterministic Black-Scholes-Merton model for 1st and 2nd order Greeks.
    /// Pure function using f64 precision.
    pub fn calculate_greeks(
        is_call: bool,
        spot: f64,
        strike: f64,
        time_to_maturity_years: f64,
        rfr: f64,
        iv: f64,
    ) -> GreeksProfile {
        let n = Normal::new(0.0, 1.0).unwrap();
        
        let sqrt_t = time_to_maturity_years.sqrt();
        let d1 = ((spot / strike).ln() + (rfr + (iv * iv) / 2.0) * time_to_maturity_years) / (iv * sqrt_t);
        let d2 = d1 - iv * sqrt_t;
        
        let nd1 = n.cdf(d1);
        let nd2 = n.cdf(d2);
        let n_prime_d1 = (-d1 * d1 / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt();
        
        let exp_rt = (-rfr * time_to_maturity_years).exp();

        if is_call {
            GreeksProfile {
                delta: nd1,
                gamma: n_prime_d1 / (spot * iv * sqrt_t),
                theta: -(spot * n_prime_d1 * iv) / (2.0 * sqrt_t) - rfr * strike * exp_rt * nd2,
                vega: spot * sqrt_t * n_prime_d1,
                rho: strike * time_to_maturity_years * exp_rt * nd2,
            }
        } else {
            GreeksProfile {
                delta: nd1 - 1.0,
                gamma: n_prime_d1 / (spot * iv * sqrt_t),
                theta: -(spot * n_prime_d1 * iv) / (2.0 * sqrt_t) + rfr * strike * exp_rt * n.cdf(-d2),
                vega: spot * sqrt_t * n_prime_d1,
                rho: -strike * time_to_maturity_years * exp_rt * n.cdf(-d2),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greeks_determinism() {
        let greeks1 = GreeksEngine::calculate_greeks(
            true,
            50_000.0,
            50_000.0,
            0.25,
            0.05,
            0.5
        );
        
        let greeks2 = GreeksEngine::calculate_greeks(
            true,
            50_000.0,
            50_000.0,
            0.25,
            0.05,
            0.5
        );
        
        assert_eq!(greeks1.delta, greeks2.delta);
        assert_eq!(greeks1.gamma, greeks2.gamma);
    }
}
