use std::sync::atomic::{AtomicU64, Ordering};

pub struct TelemetryBridge {
    // Non-deterministic wall-clock metrics
    pub process_cpu: AtomicU64,
    pub process_memory: AtomicU64,
    pub network_latency_ms: AtomicU64,
    pub websocket_reconnects: AtomicU64,
}

impl Default for TelemetryBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryBridge {
    pub fn new() -> Self {
        Self {
            process_cpu: AtomicU64::new(0),
            process_memory: AtomicU64::new(0),
            network_latency_ms: AtomicU64::new(0),
            websocket_reconnects: AtomicU64::new(0),
        }
    }

    pub fn record_network_latency(&self, ms: u64) {
        self.network_latency_ms.store(ms, Ordering::Relaxed);
    }

    pub fn record_reconnect(&self) {
        self.websocket_reconnects.fetch_add(1, Ordering::Relaxed);
    }
}
