pub fn simulate_liquidity_evaporation(base_liquidity: u64, evaporation_ppm: u64) -> u64 {
    let evaporation = (base_liquidity * evaporation_ppm) / 1_000_000;
    base_liquidity.saturating_sub(evaporation)
}
