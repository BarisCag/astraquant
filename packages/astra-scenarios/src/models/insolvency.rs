pub fn propagate_insolvency(base_deficit: i64, cascade_depth: u64, multiplier_ppm: u64) -> i64 {
    let mut deficit = base_deficit;
    for _ in 0..cascade_depth {
        deficit = deficit.saturating_add((deficit * multiplier_ppm as i64) / 1_000_000);
    }
    deficit
}
