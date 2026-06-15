use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeterministicLcg {
    state: u64,
}

impl DeterministicLcg {
    pub fn new(seed: u64) -> Self {
        // Prevent seed of 0 since some LCGs get stuck, though we add a constant.
        Self {
            state: seed.wrapping_add(1),
        }
    }

    /// Linear Congruential Generator using MMIX parameters by Donald Knuth.
    /// multiplier: 6364136223846793005, increment: 1442695040888963407, modulus: 2^64
    pub fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    /// Generate an integer in range [0, bound)
    pub fn next_bounded(&mut self, bound: u64) -> u64 {
        if bound == 0 {
            return 0;
        }
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let r = self.next_u64();
            if r >= threshold {
                return r % bound;
            }
        }
    }

    /// Generate a boolean with given probability defined in parts per million.
    pub fn next_bool_ppm(&mut self, probability_ppm: u64) -> bool {
        self.next_bounded(1_000_000) < probability_ppm
    }
}
