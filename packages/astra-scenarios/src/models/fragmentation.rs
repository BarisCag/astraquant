pub fn venue_fragmentation_penalty(venue_latency: u64, offline: bool) -> u64 {
    if offline {
        u64::MAX // Infinite latency/penalty
    } else {
        venue_latency
    }
}
