use astra_scenarios::scenario::ExperimentParameterSet;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SweepDimension {
    LiquidityCollapseSeverity {
        start_ppm: u64,
        end_ppm: u64,
        steps: u64,
    },
    MarginMultiplier {
        start_ppm: u64,
        end_ppm: u64,
        steps: u64,
    },
    VenueLatency {
        start_seq: u64,
        end_seq: u64,
        steps: u64,
    },
    CollateralHaircut {
        start_ppm: u64,
        end_ppm: u64,
        steps: u64,
    },
    StressSeverity {
        start_ppm: u64,
        end_ppm: u64,
        steps: u64,
    },
    RecoveryDelay {
        start_seq: u64,
        end_seq: u64,
        steps: u64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParameterSweep {
    pub dimensions: Vec<SweepDimension>,
}

#[derive(Clone, Debug)]
pub struct SweepExecutionPlan {
    pub parameter_sets: Vec<ExperimentParameterSet>,
}

impl ParameterSweep {
    pub fn new(dimensions: Vec<SweepDimension>) -> Self {
        Self { dimensions }
    }

    pub fn generate_plan(&self, base_parameters: &ExperimentParameterSet) -> SweepExecutionPlan {
        let mut parameter_sets = vec![base_parameters.clone()];

        for dimension in &self.dimensions {
            let mut new_sets = Vec::new();

            for base_set in &parameter_sets {
                match dimension {
                    SweepDimension::LiquidityCollapseSeverity {
                        start_ppm,
                        end_ppm,
                        steps,
                    } => {
                        let step_size = if *steps > 1 {
                            (end_ppm - start_ppm) / (steps - 1)
                        } else {
                            0
                        };
                        for i in 0..*steps {
                            let mut new_set = base_set.clone();
                            new_set.liquidity_drop_ppm = start_ppm + (i * step_size);
                            new_sets.push(new_set);
                        }
                    }
                    SweepDimension::MarginMultiplier {
                        start_ppm,
                        end_ppm,
                        steps,
                    } => {
                        let step_size = if *steps > 1 {
                            (end_ppm - start_ppm) / (steps - 1)
                        } else {
                            0
                        };
                        for i in 0..*steps {
                            let mut new_set = base_set.clone();
                            new_set.margin_multiplier_ppm = start_ppm + (i * step_size);
                            new_sets.push(new_set);
                        }
                    }
                    SweepDimension::VenueLatency {
                        start_seq,
                        end_seq,
                        steps,
                    } => {
                        let step_size = if *steps > 1 {
                            (end_seq - start_seq) / (steps - 1)
                        } else {
                            0
                        };
                        for i in 0..*steps {
                            let mut new_set = base_set.clone();
                            new_set.venue_latency_sequences = start_seq + (i * step_size);
                            new_sets.push(new_set);
                        }
                    }
                    SweepDimension::CollateralHaircut {
                        start_ppm,
                        end_ppm,
                        steps,
                    } => {
                        let step_size = if *steps > 1 {
                            (end_ppm - start_ppm) / (steps - 1)
                        } else {
                            0
                        };
                        for i in 0..*steps {
                            let mut new_set = base_set.clone();
                            new_set.collateral_haircut_ppm = start_ppm + (i * step_size);
                            new_sets.push(new_set);
                        }
                    }
                    SweepDimension::StressSeverity {
                        start_ppm,
                        end_ppm,
                        steps,
                    } => {
                        let step_size = if *steps > 1 {
                            (end_ppm - start_ppm) / (steps - 1)
                        } else {
                            0
                        };
                        for i in 0..*steps {
                            let mut new_set = base_set.clone();
                            new_set.stress_severity_ppm = start_ppm + (i * step_size);
                            new_sets.push(new_set);
                        }
                    }
                    SweepDimension::RecoveryDelay {
                        start_seq,
                        end_seq,
                        steps,
                    } => {
                        let step_size = if *steps > 1 {
                            (end_seq - start_seq) / (steps - 1)
                        } else {
                            0
                        };
                        for i in 0..*steps {
                            let mut new_set = base_set.clone();
                            new_set.recovery_delay_sequences = start_seq + (i * step_size);
                            new_sets.push(new_set);
                        }
                    }
                }
            }
            parameter_sets = new_sets;
        }

        SweepExecutionPlan { parameter_sets }
    }
}

pub struct SweepIterator {
    plan: SweepExecutionPlan,
    current_index: usize,
}

impl SweepIterator {
    pub fn new(plan: SweepExecutionPlan) -> Self {
        Self {
            plan,
            current_index: 0,
        }
    }
}

impl Iterator for SweepIterator {
    type Item = ExperimentParameterSet;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.plan.parameter_sets.len() {
            let item = self.plan.parameter_sets[self.current_index].clone();
            self.current_index += 1;
            Some(item)
        } else {
            None
        }
    }
}
