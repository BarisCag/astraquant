use crate::telemetry::TelemetryBridge;
use std::sync::Arc;

pub struct MetricsExporter {
    pub telemetry: Arc<TelemetryBridge>,
}

impl MetricsExporter {
    pub fn new(telemetry: Arc<TelemetryBridge>) -> Self {
        Self { telemetry }
    }

    /// Renders Prometheus text-format metrics
    pub fn render_prometheus(&self) -> String {
        let mut buffer = String::new();

        // Metrics rendered from actual TelemetryBridge state

        buffer.push_str(
            "# HELP astra_websocket_reconnects Number of physical exchange disconnects\n",
        );
        buffer.push_str("# TYPE astra_websocket_reconnects counter\n");
        buffer.push_str(&format!(
            "astra_websocket_reconnects {}\n",
            self.telemetry
                .websocket_reconnects
                .load(std::sync::atomic::Ordering::Relaxed)
        ));

        buffer.push_str("# HELP astra_network_latency_ms Wall-clock network RTT\n");
        buffer.push_str("# TYPE astra_network_latency_ms gauge\n");
        buffer.push_str(&format!(
            "astra_network_latency_ms {}\n",
            self.telemetry
                .network_latency_ms
                .load(std::sync::atomic::Ordering::Relaxed)
        ));

        buffer
    }
}
