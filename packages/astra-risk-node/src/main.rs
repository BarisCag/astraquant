pub mod analytics;
pub mod exporter;
pub mod greeks;
pub mod var_calc;

use astra_core::journal::EventJournal;
use crate::analytics::RiskAnalyticsEngine;
use crate::exporter::MetricsExporter;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let exporter = MetricsExporter::new();
    let exporter_clone = exporter.clone();

    // Spawn the metrics server on port 9090
    tokio::spawn(async move {
        exporter_clone.start_server().await;
    });

    let mut analytics = RiskAnalyticsEngine::new(exporter);

    let journal_path = "journal.astra";
    let mut last_seq = 0;

    println!("Starting astra-risk-node...");
    println!("Listening for Prometheus metrics on http://0.0.0.0:9090/metrics");

    loop {
        if std::path::Path::new(journal_path).exists() {
            if let Ok(journal) = EventJournal::open(journal_path) {
                if let Ok(iter) = journal.iter_from(last_seq) {
                    for event_res in iter {
                        if let Ok(event) = event_res {
                            analytics.process_event(&event);
                            last_seq = event.sequence_id;
                        }
                    }
                }
            }
        }
        sleep(Duration::from_millis(50)).await;
    }
}
