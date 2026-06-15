pub fn determine_cascade_depth(base_depth: u64, stress_score_ppm: u64) -> u64 {
    base_depth + (stress_score_ppm / 100_000)
}
