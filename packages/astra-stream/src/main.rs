//! astra-stream v0.2.1 CLI: deterministic market data ingestion and replay.
//!
//! # Usage
//!
//! ```text
//! astra-stream ingest --symbol btcusdt [--symbol ethusdt ...] \
//!              [--journal-dir ./data/stream] [--metrics-port 9090]
//!
//! astra-stream replay <journal-file.astra_jl>
//! astra-stream replay ./journals/              # directory replay
//! astra-stream replay <file> --bench-csv bench.csv
//! ```

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(
    name = "astra-stream",
    about = "Deterministic market data ingestion and replay",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Ingest live trades from Binance spot and journal them (supports multiple symbols)
    Ingest {
        /// Trading pair symbol(s). Specify once per symbol: --symbol btcusdt --symbol ethusdt
        #[arg(long, required = true)]
        symbol: Vec<String>,

        /// Directory for journal files (one rotating file per symbol per hour)
        #[arg(long, default_value = "./data/stream")]
        journal_dir: String,

        /// Port for Prometheus metrics HTTP endpoint
        #[arg(long, default_value_t = 9090)]
        metrics_port: u16,
    },

    /// Replay a journal file or directory and verify deterministic hashes
    Replay {
        /// Path to a .astra_jl file or a directory containing .astra_jl files
        path: PathBuf,

        /// Optional: export benchmark summary to a CSV file
        #[arg(long)]
        bench_csv: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Ingest {
            symbol,
            journal_dir,
            metrics_port,
        } => run_ingest(symbol, journal_dir, metrics_port),
        Command::Replay { path, bench_csv } => run_replay(path, bench_csv),
    }
}

fn run_ingest(symbols: Vec<String>, journal_dir: String, metrics_port: u16) {
    eprintln!(
        "[astra-stream] mode=ingest symbols=[{}] journal_dir={}",
        symbols.join(", "),
        journal_dir
    );

    std::fs::create_dir_all(&journal_dir).expect("failed to create journal directory");

    let metrics = Arc::new(astra_stream::metrics::StreamMetrics::new());

    // Spawn Prometheus metrics server on a background thread
    let metrics_clone = metrics.clone();
    std::thread::spawn(move || {
        astra_stream::metrics::serve_metrics(metrics_clone, metrics_port);
    });

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    if symbols.len() == 1 {
        // Single-symbol path: simpler, uses JournalRotator directly
        let symbol = symbols.into_iter().next().unwrap();
        rt.block_on(async {
            let mut rotator = astra_stream::rotation::JournalRotator::open(&journal_dir, &symbol)
                .expect("failed to open journal rotator");
            eprintln!(
                "[astra-stream] journal: {} (seq={})",
                rotator.current_path().display(),
                rotator.journal().unwrap().next_sequence_id()
            );
            let stream = astra_stream::ingestion::BinanceTradeStream::new(symbol, metrics);
            if let Err(e) = stream.run(&mut rotator).await {
                eprintln!("[astra-stream] fatal ingestion error: {}", e);
                std::process::exit(1);
            }
        });
    } else {
        // Multi-symbol path: per-symbol channels, unified journal writer
        rt.block_on(async {
            let ingestion =
                astra_stream::ingestion::MultiSymbolIngestion::new(symbols, journal_dir, metrics);
            if let Err(e) = ingestion.run().await {
                eprintln!("[astra-stream] fatal ingestion error: {}", e);
                std::process::exit(1);
            }
        });
    }
}

fn run_replay(path: PathBuf, bench_csv: Option<PathBuf>) {
    eprintln!("[astra-stream] mode=replay path={}", path.display());

    if !path.exists() {
        eprintln!("[astra-stream] error: path not found: {}", path.display());
        std::process::exit(1);
    }

    let report = match astra_stream::replay::StreamReplayEngine::replay(&path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[astra-stream] replay error: {}", e);
            std::process::exit(1);
        }
    };

    // Print structured replay report
    eprintln!("═══════════════════════════════════════════════════════");
    eprintln!("  AstraQuant Stream Replay Report  [v0.2.1]");
    eprintln!("═══════════════════════════════════════════════════════");
    if !report.files_replayed.is_empty() {
        eprintln!("  files_replayed  : {}", report.files_replayed.len());
        for f in &report.files_replayed {
            eprintln!(
                "                    {}",
                f.file_name().unwrap_or_default().to_string_lossy()
            );
        }
    }
    eprintln!("  events_replayed : {}", report.events_replayed);
    eprintln!(
        "  replay_hash     : {}",
        astra_core::hash_to_hex(&report.final_hash)
    );
    eprintln!("  elapsed         : {} μs", report.elapsed_us);
    eprintln!("  throughput      : {} events/sec", report.events_per_sec());

    if report.violations.is_empty() {
        eprintln!("  integrity       : PASS (no violations)");
    } else {
        eprintln!(
            "  integrity       : FAIL ({} violation(s))",
            report.violations.len()
        );
        for (i, v) in report.violations.iter().enumerate() {
            print_violation(i + 1, v);
        }
    }
    eprintln!("═══════════════════════════════════════════════════════");

    // Optional CSV export
    if let Some(csv_path) = bench_csv {
        match astra_stream::replay::export_csv(&report, &csv_path) {
            Ok(()) => eprintln!(
                "[astra-stream] benchmark CSV written to {}",
                csv_path.display()
            ),
            Err(e) => eprintln!("[astra-stream] CSV export error: {}", e),
        }
    }

    if !report.violations.is_empty() {
        std::process::exit(2);
    }
}

fn print_violation(n: usize, v: &astra_stream::replay::IntegrityViolation) {
    use astra_stream::replay::IntegrityViolation::*;
    match v {
        DuplicateTradeId {
            trade_id,
            sequence_id,
        } => {
            eprintln!(
                "    [{}] DUPLICATE_TRADE_ID trade_id={} at seq={}",
                n, trade_id, sequence_id
            );
        }
        OutOfOrderTimestamp {
            sequence_id,
            prev_timestamp_us,
            curr_timestamp_us,
        } => {
            eprintln!(
                "    [{}] OUT_OF_ORDER seq={} prev_ts={} curr_ts={}",
                n, sequence_id, prev_timestamp_us, curr_timestamp_us
            );
        }
        SequenceGap {
            expected_sequence_id,
            actual_sequence_id,
        } => {
            eprintln!(
                "    [{}] SEQUENCE_GAP expected={} actual={}",
                n, expected_sequence_id, actual_sequence_id
            );
        }
        ReconnectGap {
            sequence_id,
            prev_trade_id,
            curr_trade_id,
        } => {
            eprintln!(
                "    [{}] RECONNECT_GAP seq={} prev_trade_id={} curr_trade_id={} (gap={})",
                n,
                sequence_id,
                prev_trade_id,
                curr_trade_id,
                curr_trade_id
                    .saturating_sub(*prev_trade_id)
                    .saturating_sub(1)
            );
        }
        StreamDiscontinuity {
            sequence_id,
            prev_timestamp_us,
            curr_timestamp_us,
            gap_us,
        } => {
            eprintln!(
                "    [{}] STREAM_DISCONTINUITY seq={} prev_ts={} curr_ts={} gap={}μs ({:.1}s)",
                n,
                sequence_id,
                prev_timestamp_us,
                curr_timestamp_us,
                gap_us,
                *gap_us as f64 / 1_000_000.0
            );
        }
    }
}
