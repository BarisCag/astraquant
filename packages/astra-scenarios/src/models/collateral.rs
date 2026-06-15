pub fn simulate_haircut_stress(base_haircut_ppm: u64, stress_severity_ppm: u64) -> u64 {
    // Haircut drops proportionally to stress
    let drop = (base_haircut_ppm * stress_severity_ppm) / 1_000_000;
    base_haircut_ppm.saturating_sub(drop)
}
