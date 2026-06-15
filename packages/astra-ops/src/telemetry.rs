use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;

/// Shared, thread-safe operational metrics bridge.
/// Non-deterministic wall-clock metrics only — never enters astra-core.
pub struct OperationalTelemetry {
    /// Blake3 state hash as a signed i64 (first 8 bytes, big-endian cast).
    /// Signed to satisfy Prometheus gauge type semantics.
    pub kernel_state_hash: AtomicI64,
    /// Cumulative WASM gas consumed across all sandboxes.
    pub wasm_gas_consumed: AtomicU64,
    /// Total events processed by the kernel since boot.
    pub total_events_processed: AtomicU64,
    /// Current outstanding messages in the actor mailbox queue.
    pub mailbox_depth: AtomicU64,
}

impl OperationalTelemetry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            kernel_state_hash: AtomicI64::new(0),
            wasm_gas_consumed: AtomicU64::new(0),
            total_events_processed: AtomicU64::new(0),
            mailbox_depth: AtomicU64::new(0),
        })
    }

    /// Updates the kernel state hash gauge from a raw Blake3 [u8; 32].
    /// Casts the first 8 bytes as a big-endian i64 for Prometheus compatibility.
    pub fn update_kernel_hash(&self, hash: &[u8; 32]) {
        let val = i64::from_be_bytes(hash[..8].try_into().unwrap_or([0u8; 8]));
        self.kernel_state_hash.store(val, Ordering::Release);
    }

    pub fn increment_events(&self) {
        self.total_events_processed.fetch_add(1, Ordering::Release);
    }

    pub fn add_gas(&self, gas: u64) {
        self.wasm_gas_consumed.fetch_add(gas, Ordering::Release);
    }

    pub fn set_mailbox_depth(&self, depth: u64) {
        self.mailbox_depth.store(depth, Ordering::Release);
    }

    /// Renders all metrics in Prometheus text exposition format (version 0.0.4).
    pub fn render_prometheus(&self) -> String {
        let hash_val = self.kernel_state_hash.load(Ordering::Acquire);
        let gas_val = self.wasm_gas_consumed.load(Ordering::Acquire);
        let events_val = self.total_events_processed.load(Ordering::Acquire);
        let mailbox_val = self.mailbox_depth.load(Ordering::Acquire);

        format!(
            "# HELP astra_kernel_state_hash Blake3 state hash first-8-bytes cast to i64\n\
             # TYPE astra_kernel_state_hash gauge\n\
             astra_kernel_state_hash {hash_val}\n\
             # HELP astra_wasm_gas_consumed Total WASM gas consumed by sandboxed strategies\n\
             # TYPE astra_wasm_gas_consumed counter\n\
             astra_wasm_gas_consumed {gas_val}\n\
             # HELP astra_total_events_processed Total events applied by the AstraKernel since boot\n\
             # TYPE astra_total_events_processed counter\n\
             astra_total_events_processed {events_val}\n\
             # HELP astra_mailbox_depth Current depth of the actor mailbox queue\n\
             # TYPE astra_mailbox_depth gauge\n\
             astra_mailbox_depth {mailbox_val}\n"
        )
    }
}
