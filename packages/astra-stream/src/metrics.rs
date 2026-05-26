//! Prometheus-compatible metrics for the astra-stream subsystem.
//!
//! All counters use atomic operations for lock-free thread-safe access.
//! The HTTP endpoint serves metrics in Prometheus text exposition format
//! via a bare `TcpListener` — no external HTTP framework.
//!
//! # Memory model
//!
//! All metrics state fits in a fixed number of `AtomicU64` fields.
//! No dynamic allocation occurs during metric recording or rendering.

use std::io::Write;
use std::net::TcpListener;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Atomic metrics counters for the stream subsystem.
pub struct StreamMetrics {
    /// Total events ingested and journaled.
    pub events_ingested: AtomicU64,
    /// Total WebSocket reconnections.
    pub websocket_reconnects: AtomicU64,
    /// Total journal bytes written (payload + length prefix).
    pub journal_bytes_written: AtomicU64,
    /// Total events replayed (set after replay completes).
    pub replay_events: AtomicU64,
    /// Replay throughput in events/sec (gauge, set after replay completes).
    pub replay_throughput_eps: AtomicU64,
    /// Last journal write latency in microseconds (gauge).
    pub last_write_latency_us: AtomicU64,
    /// Total integrity violations detected (counter).
    pub integrity_violations_total: AtomicU64,
    /// Process start time for computing events/sec gauge.
    start_instant: Instant,
}

impl StreamMetrics {
    pub fn new() -> Self {
        Self {
            events_ingested: AtomicU64::new(0),
            websocket_reconnects: AtomicU64::new(0),
            journal_bytes_written: AtomicU64::new(0),
            replay_events: AtomicU64::new(0),
            replay_throughput_eps: AtomicU64::new(0),
            last_write_latency_us: AtomicU64::new(0),
            integrity_violations_total: AtomicU64::new(0),
            start_instant: Instant::now(),
        }
    }

    /// Record a single ingested event.
    pub fn record_event(&self) {
        self.events_ingested.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a WebSocket reconnection.
    pub fn record_reconnect(&self) {
        self.websocket_reconnects.fetch_add(1, Ordering::Relaxed);
    }

    /// Record journal bytes written.
    pub fn record_journal_bytes(&self, bytes: u64) {
        self.journal_bytes_written
            .fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record the latency of the most recent journal write in microseconds.
    pub fn record_write_latency(&self, us: u64) {
        self.last_write_latency_us.store(us, Ordering::Relaxed);
    }

    /// Record an integrity violation (any type).
    pub fn record_integrity_violation(&self) {
        self.integrity_violations_total
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Set replay statistics after a replay completes.
    pub fn set_replay_stats(&self, events: u64, eps: u64) {
        self.replay_events.store(events, Ordering::Relaxed);
        self.replay_throughput_eps.store(eps, Ordering::Relaxed);
    }

    /// Compute events/sec as integer (total events / elapsed seconds).
    pub fn events_per_sec(&self) -> u64 {
        let elapsed_secs = self.start_instant.elapsed().as_secs();
        if elapsed_secs == 0 {
            return 0;
        }
        self.events_ingested.load(Ordering::Relaxed) / elapsed_secs
    }

    /// Compute bytes/sec written as integer (total bytes / elapsed seconds).
    pub fn bytes_per_sec(&self) -> u64 {
        let elapsed_secs = self.start_instant.elapsed().as_secs();
        if elapsed_secs == 0 {
            return 0;
        }
        self.journal_bytes_written.load(Ordering::Relaxed) / elapsed_secs
    }

    /// Render all metrics in Prometheus text exposition format.
    pub fn render_prometheus(&self) -> String {
        let mut buf = String::with_capacity(2048);

        write_metric(
            &mut buf,
            "astra_stream_events_ingested_total",
            "counter",
            "Total events received and journaled",
            self.events_ingested.load(Ordering::Relaxed),
        );
        write_metric(
            &mut buf,
            "astra_stream_events_per_second",
            "gauge",
            "Current ingestion rate (events/sec, integer)",
            self.events_per_sec(),
        );
        write_metric(
            &mut buf,
            "astra_stream_websocket_reconnects_total",
            "counter",
            "Total WebSocket reconnect count",
            self.websocket_reconnects.load(Ordering::Relaxed),
        );
        write_metric(
            &mut buf,
            "astra_stream_journal_bytes_written_total",
            "counter",
            "Total journal bytes written including length prefixes",
            self.journal_bytes_written.load(Ordering::Relaxed),
        );
        write_metric(
            &mut buf,
            "astra_stream_bytes_per_second",
            "gauge",
            "Current journal write rate (bytes/sec, integer)",
            self.bytes_per_sec(),
        );
        write_metric(
            &mut buf,
            "astra_stream_last_write_latency_us",
            "gauge",
            "Most recent journal write latency in microseconds",
            self.last_write_latency_us.load(Ordering::Relaxed),
        );
        write_metric(
            &mut buf,
            "astra_stream_integrity_violations_total",
            "counter",
            "Total integrity violations detected during ingestion",
            self.integrity_violations_total.load(Ordering::Relaxed),
        );
        write_metric(
            &mut buf,
            "astra_stream_replay_events_total",
            "counter",
            "Events replayed (set after replay completes)",
            self.replay_events.load(Ordering::Relaxed),
        );
        write_metric(
            &mut buf,
            "astra_stream_replay_throughput_eps",
            "gauge",
            "Replay throughput in events/sec (set after replay completes)",
            self.replay_throughput_eps.load(Ordering::Relaxed),
        );

        buf
    }
}

impl Default for StreamMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Write a single Prometheus metric entry to the buffer.
fn write_metric(buf: &mut String, name: &str, metric_type: &str, help: &str, value: u64) {
    buf.push_str(&format!("# HELP {} {}\n", name, help));
    buf.push_str(&format!("# TYPE {} {}\n", name, metric_type));
    buf.push_str(&format!("{} {}\n", name, value));
}

/// Serve Prometheus metrics over HTTP on the given port.
/// This function blocks forever, serving one request at a time.
pub fn serve_metrics(metrics: Arc<StreamMetrics>, port: u16) {
    let listener = match TcpListener::bind(("0.0.0.0", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[astra-stream] metrics bind failed on port {}: {}", port, e);
            return;
        }
    };
    eprintln!("[astra-stream] Prometheus metrics on 0.0.0.0:{}", port);

    for stream in listener.incoming().flatten() {
        let mut stream = stream;
        let body = metrics.render_prometheus();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes());
    }
}
