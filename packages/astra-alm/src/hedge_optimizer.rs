use crate::types::{ALMMismatchReport, HedgeRatio};

pub struct HedgeOptimizer {
    learning_rate: f64,
    max_iterations: u32,
    convergence_eps: f64,
}

impl HedgeOptimizer {
    pub fn new() -> Self {
        Self {
            learning_rate: 0.01,
            max_iterations: 1000,
            convergence_eps: 1e-6,
        }
    }

    pub fn optimize(
        &self,
        mismatches: &[ALMMismatchReport],
        base_cvar: f64,
    ) -> Vec<HedgeRatio> {
        let mut ratios: Vec<HedgeRatio> = mismatches
            .iter()
            .enumerate()
            .map(|(i, _)| HedgeRatio {
                instrument: format!("HEDGE_INSTRUMENT_{}", i),
                ratio: 0.5,
                notional: 0,
            })
            .collect();

        for _ in 0..self.max_iterations {
            let current_cvar = Self::compute_portfolio_cvar(&ratios, mismatches, base_cvar);
            let mut max_diff = 0.0_f64;

            let mut gradients = Vec::with_capacity(ratios.len());

            for i in 0..ratios.len() {
                let mut perturbed = ratios.clone();
                perturbed[i].ratio += 1e-4;
                let cvar_plus = Self::compute_portfolio_cvar(&perturbed, mismatches, base_cvar);
                let gradient = (cvar_plus - current_cvar) / 1e-4;
                gradients.push(gradient);
            }

            for i in 0..ratios.len() {
                let diff = self.learning_rate * gradients[i];
                let old_ratio = ratios[i].ratio;
                
                ratios[i].ratio = (ratios[i].ratio - diff).clamp(0.0, 1.0);
                
                let step_diff = (ratios[i].ratio - old_ratio).abs();
                if step_diff > max_diff {
                    max_diff = step_diff;
                }
            }

            if max_diff < self.convergence_eps {
                break;
            }
        }

        ratios
    }

    pub fn compute_portfolio_cvar(
        ratios: &[HedgeRatio],
        mismatches: &[ALMMismatchReport],
        base_cvar: f64,
    ) -> f64 {
        let mut penalty = 0.0;
        for (i, m) in mismatches.iter().enumerate() {
            let ratio = ratios.get(i).map(|r| r.ratio).unwrap_or(0.0);
            // Simple proxy for risk: unhedged gap squared
            let unhedged_factor = 1.0 - ratio;
            penalty += m.duration_gap.abs() * unhedged_factor * unhedged_factor;
            // Add a small penalty to keep ratio from just zooming to 1.0 if not needed
            penalty += ratio * 0.1;
        }
        base_cvar + penalty * 100.0
    }
}

impl Default for HedgeOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
