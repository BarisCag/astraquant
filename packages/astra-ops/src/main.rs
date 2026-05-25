//! Ops daemon: journal replay/seed and Prometheus text metrics (research prototype).

use std::env;
use std::fs;
use std::io::Write;
use std::net::TcpListener;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use astra_core::events::{EventType, PayloadMetadata};
use astra_core::hashing::{hash_to_hex, DeterministicState};
use astra_core::journal::EventJournal;
use astra_core::kernel::AstraKernel;
use astra_core::replay::{EventReducer, ReplayEngine};

use astra_ops::metrics_exporter::MetricsExporter;
use astra_ops::telemetry::TelemetryBridge;

fn serve_prometheus(exporter: Arc<MetricsExporter>, port: u16) {
    let listener = match TcpListener::bind(("0.0.0.0", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("metrics bind failed on port {}: {}", port, e);
            return;
        }
    };
    eprintln!("Prometheus metrics on 0.0.0.0:{}", port);

    for mut stream in listener.incoming().flatten() {
        let body = exporter.render_prometheus();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes());
    }
}

fn main() {
    let journal_dir =
        env::var("ASTRA_JOURNAL_DIR").unwrap_or_else(|_| "./data/journal".to_string());
    let metrics_port: u16 = env::var("ASTRA_HTTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8081);

    let telemetry = Arc::new(TelemetryBridge::new());
    let exporter = Arc::new(MetricsExporter::new(telemetry));
    thread::spawn({
        let exporter = exporter.clone();
        move || serve_prometheus(exporter, metrics_port)
    });

    fs::create_dir_all(&journal_dir).expect("create journal dir");

    let has_journals = fs::read_dir(&journal_dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| e.path().extension().and_then(|s| s.to_str()) == Some("astra_jl"));

    let limits = astra_core::risk::RiskLimits::new(
        astra_core::types::Money::new(100_000_000),
        astra_core::types::Quantity::new(1_000),
    );

    if has_journals {
        eprintln!("recovery: replaying journal");
        let mut kernel = AstraKernel::new(astra_core::runtime::StrategyRuntime::new(
            astra_core::exchange::ExchangeRuntime::new(limits),
        ));
        let path = Path::new(&journal_dir).join("journal.astra_jl");
        let journal = EventJournal::open(&path).expect("open journal");
        let result = ReplayEngine::replay_journal(&journal, &mut kernel).expect("replay");
        eprintln!(
            "state_hash={} events={} seq={}",
            hash_to_hex(&result.final_state_hash),
            result.events_applied,
            result.final_sequence_id
        );
    } else {
        eprintln!("seed: writing 10000 market ticks");
        let path = Path::new(&journal_dir).join("journal.astra_jl");
        let mut journal = EventJournal::open(&path).expect("open journal");
        let mut kernel = AstraKernel::new(astra_core::runtime::StrategyRuntime::new(
            astra_core::exchange::ExchangeRuntime::new(limits),
        ));
        for _ in 0..10_000 {
            let event = journal
                .commit(0, EventType::MarketTick, vec![], PayloadMetadata::raw())
                .expect("commit");
            kernel.apply(&event).expect("apply");
        }
        eprintln!(
            "state_hash={} events={}",
            hash_to_hex(&kernel.state_hash()),
            journal.len()
        );
    }

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
