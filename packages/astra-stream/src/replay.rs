//! Deterministic journal replay and integrity verification.
//!
//! The [`StreamReplayEngine`] reads a journal file (or directory of journal files),
//! deserializes each event's payload as a [`NormalizedMarketEvent`], runs integrity
//! checks, and builds a deterministic BLAKE3 hash chain. The final replay hash is
//! identical for any two replays of the same journal(s), providing cryptographic
//! proof of determinism.
//!
//! # Directory replay
//!
//! When replaying a directory, all `.astra_jl` files are sorted lexicographically
//! by filename before replay. The lexicographic sort is stable and deterministic
//! across platforms because filenames follow the format:
//!
//! ```text
//! {symbol}_{YYYY}_{MM}_{DD}_{HH}.astra_jl
//! ```
//!
//! This means chronological order equals lexicographic order for same-symbol files.
//! The running hash accumulates across all files in sorted order.

use crate::normalized::NormalizedMarketEvent;
use astra_core::events::AstraEvent;
use astra_core::hashing::{hash_bytes, DeterministicState};
use astra_core::journal::EventJournal;
use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Report produced by a journal replay (single file or directory).
#[derive(Debug)]
pub struct ReplayReport {
    /// Number of events successfully replayed.
    pub events_replayed: u64,
    /// BLAKE3 hash chain over all replayed event state hashes (deterministic).
    pub final_hash: [u8; 32],
    /// Wall-clock replay duration in microseconds (non-deterministic, for benchmarking).
    pub elapsed_us: u64,
    /// Integrity violations detected during replay.
    pub violations: Vec<IntegrityViolation>,
    /// Journal files replayed in order (for directory mode).
    pub files_replayed: Vec<PathBuf>,
}

impl ReplayReport {
    /// Compute replay throughput as integer events per second.
    pub fn events_per_sec(&self) -> u64 {
        if self.elapsed_us == 0 {
            return 0;
        }
        self.events_replayed
            .saturating_mul(1_000_000)
            .checked_div(self.elapsed_us)
            .unwrap_or(0)
    }
}

/// Types of integrity violations detected during replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityViolation {
    /// A `trade_id` appeared more than once in the journal.
    DuplicateTradeId { trade_id: u64, sequence_id: u64 },
    /// An event's exchange timestamp is earlier than the previous event's.
    OutOfOrderTimestamp {
        sequence_id: u64,
        prev_timestamp_us: u64,
        curr_timestamp_us: u64,
    },
    /// A gap in the journal `sequence_id` (expected N, got M where M ≠ N).
    SequenceGap {
        expected_sequence_id: u64,
        actual_sequence_id: u64,
    },
    /// A `trade_id` jumped by more than 1 between consecutive events,
    /// indicating a possible reconnect gap where trades were missed.
    ReconnectGap {
        sequence_id: u64,
        prev_trade_id: u64,
        curr_trade_id: u64,
    },
    /// A large gap in exchange timestamps within a single stream.
    StreamDiscontinuity {
        sequence_id: u64,
        prev_timestamp_us: u64,
        curr_timestamp_us: u64,
        gap_us: u64,
    },
}

/// Threshold for stream discontinuity warnings: 60 seconds in microseconds.
const DISCONTINUITY_THRESHOLD_US: u64 = 60_000_000;

/// Deterministic journal replay engine with integrity verification.
///
/// Reads a journal file (or directory of journal files) produced by the ingestion
/// subsystem, deserializes each event's payload as a [`NormalizedMarketEvent`],
/// and builds a running BLAKE3 hash chain: `H(prev_hash || event.state_hash())`.
///
/// The final hash is fully deterministic — replaying the same journal(s) always
/// produces the same hash, regardless of system clock or platform.
pub struct StreamReplayEngine;

impl StreamReplayEngine {
    /// Replay a journal file or directory.
    ///
    /// If `path` is a file, replays that single file.
    /// If `path` is a directory, replays all `.astra_jl` files in lexicographic order.
    pub fn replay(path: &Path) -> io::Result<ReplayReport> {
        if path.is_dir() {
            Self::replay_directory(path)
        } else {
            Self::replay_journal(path)
        }
    }

    /// Replay a single journal file from disk.
    pub fn replay_journal(path: &Path) -> io::Result<ReplayReport> {
        let start = Instant::now();
        let file_path = path.to_path_buf();
        let mut violations = Vec::new();

        let mut report = Self::replay_file_into(
            path,
            &mut 0u64,
            &mut [0u8; 32],
            &mut BTreeSet::new(),
            &mut None,
            &mut None,
            &mut violations,
        )?;

        report.elapsed_us = start.elapsed().as_micros() as u64;
        report.files_replayed = vec![file_path];
        report.violations = violations;
        Ok(report)
    }

    /// Replay all `.astra_jl` files in a directory in deterministic lexicographic order.
    ///
    /// The hash chain accumulates across all files, so the final hash covers
    /// the entire directory's contents. Throughput is aggregated.
    pub fn replay_directory(dir: &Path) -> io::Result<ReplayReport> {
        let start = Instant::now();

        // Collect and sort files deterministically
        let mut journal_files = collect_journal_files(dir)?;
        journal_files.sort(); // lexicographic, deterministic

        if journal_files.is_empty() {
            return Ok(ReplayReport {
                events_replayed: 0,
                final_hash: [0u8; 32],
                elapsed_us: 0,
                violations: Vec::new(),
                files_replayed: Vec::new(),
            });
        }

        eprintln!(
            "[astra-stream] replaying {} journal file(s) from {}",
            journal_files.len(),
            dir.display()
        );

        // Shared state across all files for cross-file integrity checking
        let mut total_events: u64 = 0;
        let mut running_hash: [u8; 32] = [0u8; 32];
        let mut seen_trade_ids: BTreeSet<u64> = BTreeSet::new();
        let mut prev_timestamp_us: Option<u64> = None;
        let mut expected_seq: Option<u64> = None;
        let mut all_violations: Vec<IntegrityViolation> = Vec::new();

        for file in &journal_files {
            eprintln!("[astra-stream]   → {}", file.display());
            let file_report = Self::replay_file_into(
                file,
                &mut total_events,
                &mut running_hash,
                &mut seen_trade_ids,
                &mut prev_timestamp_us,
                &mut expected_seq,
                &mut all_violations,
            )?;
            total_events = file_report.events_replayed; // updated in-place by replay_file_into
        }

        let elapsed_us = start.elapsed().as_micros() as u64;

        Ok(ReplayReport {
            events_replayed: total_events,
            final_hash: running_hash,
            elapsed_us,
            violations: all_violations,
            files_replayed: journal_files,
        })
    }

    /// Internal: replay a single file, updating shared state in-place.
    /// Returns a partial ReplayReport (elapsed_us and files_replayed are unset).
    #[allow(clippy::too_many_arguments)]
    fn replay_file_into(
        path: &Path,
        events_replayed: &mut u64,
        running_hash: &mut [u8; 32],
        seen_trade_ids: &mut BTreeSet<u64>,
        prev_timestamp_us: &mut Option<u64>,
        expected_seq: &mut Option<u64>,
        violations: &mut Vec<IntegrityViolation>,
    ) -> io::Result<ReplayReport> {
        let iter = EventJournal::iter_path(path)?;
        let mut prev_trade_id: Option<u64> = None;

        for event_result in iter {
            let event: AstraEvent = event_result?;

            // --- Sequence gap detection ---
            if let Some(exp) = *expected_seq {
                if event.sequence_id != exp {
                    violations.push(IntegrityViolation::SequenceGap {
                        expected_sequence_id: exp,
                        actual_sequence_id: event.sequence_id,
                    });
                }
            }
            *expected_seq = Some(event.sequence_id + 1);

            // --- Deserialize market event payload ---
            let market_event =
                NormalizedMarketEvent::from_payload(&event.payload).map_err(|e| {
                    io::Error::other(format!(
                        "payload deserialization failed at seq {}: {}",
                        event.sequence_id, e
                    ))
                })?;

            // --- Duplicate trade_id detection ---
            if !seen_trade_ids.insert(market_event.trade_id) {
                violations.push(IntegrityViolation::DuplicateTradeId {
                    trade_id: market_event.trade_id,
                    sequence_id: event.sequence_id,
                });
            }

            // --- Reconnect gap detection: non-consecutive trade_ids ---
            if let Some(prev_id) = prev_trade_id {
                if market_event.trade_id > prev_id.saturating_add(1) {
                    violations.push(IntegrityViolation::ReconnectGap {
                        sequence_id: event.sequence_id,
                        prev_trade_id: prev_id,
                        curr_trade_id: market_event.trade_id,
                    });
                }
            }
            prev_trade_id = Some(market_event.trade_id);

            // --- Out-of-order timestamp detection ---
            if let Some(prev_ts) = *prev_timestamp_us {
                if market_event.exchange_timestamp_us < prev_ts {
                    violations.push(IntegrityViolation::OutOfOrderTimestamp {
                        sequence_id: event.sequence_id,
                        prev_timestamp_us: prev_ts,
                        curr_timestamp_us: market_event.exchange_timestamp_us,
                    });
                }
                // --- Stream discontinuity detection: large timestamp gap ---
                let gap_us = market_event.exchange_timestamp_us.saturating_sub(prev_ts);
                if gap_us > DISCONTINUITY_THRESHOLD_US {
                    violations.push(IntegrityViolation::StreamDiscontinuity {
                        sequence_id: event.sequence_id,
                        prev_timestamp_us: prev_ts,
                        curr_timestamp_us: market_event.exchange_timestamp_us,
                        gap_us,
                    });
                }
            }
            *prev_timestamp_us = Some(market_event.exchange_timestamp_us);

            // --- Deterministic hash chain: H(prev_hash || event_state_hash) ---
            let event_hash = market_event.state_hash();
            let mut chain_input = Vec::with_capacity(64);
            chain_input.extend_from_slice(running_hash);
            chain_input.extend_from_slice(&event_hash);
            *running_hash = hash_bytes(&chain_input);

            *events_replayed += 1;
        }

        Ok(ReplayReport {
            events_replayed: *events_replayed,
            final_hash: *running_hash,
            elapsed_us: 0,              // caller fills this in
            violations: Vec::new(),     // caller owns the shared violations vec
            files_replayed: Vec::new(), // caller fills this in
        })
    }
}

/// Collect all `.astra_jl` files in a directory (non-recursive).
pub fn collect_journal_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "astra_jl" {
                    files.push(path);
                }
            }
        }
    }
    Ok(files)
}

/// Export a replay report as CSV.
/// Format: file,events_replayed,elapsed_us,events_per_sec,violations
pub fn export_csv(report: &ReplayReport, path: &Path) -> io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::File::create(path)?;
    writeln!(
        f,
        "file,events_replayed,elapsed_us,events_per_sec,violations"
    )?;
    let files_str = if report.files_replayed.is_empty() {
        "<single>".to_string()
    } else {
        report
            .files_replayed
            .iter()
            .map(|p| {
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<Vec<_>>()
            .join(";")
    };
    writeln!(
        f,
        "{},{},{},{},{}",
        files_str,
        report.events_replayed,
        report.elapsed_us,
        report.events_per_sec(),
        report.violations.len()
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalized::symbol_from_str;
    use astra_core::events::{EventType, PayloadEncoding, PayloadMetadata};
    use astra_core::journal::EventJournal;

    fn make_test_event(trade_id: u64, timestamp_us: u64, price: i64) -> NormalizedMarketEvent {
        NormalizedMarketEvent {
            exchange_timestamp_us: timestamp_us,
            receive_timestamp_ns: 0,
            symbol: symbol_from_str("BTCUSDT"),
            price,
            quantity: 100_000,
            is_buyer_maker: false,
            trade_id,
        }
    }

    fn write_events_to_journal(path: &Path, events: &[NormalizedMarketEvent]) {
        let mut journal = EventJournal::create(path, 0).unwrap();
        for event in events {
            let payload = event.to_journal_payload().unwrap();
            journal
                .commit(
                    event.exchange_timestamp_us.saturating_mul(1000),
                    EventType::MarketTick,
                    payload,
                    PayloadMetadata::new(PayloadEncoding::Bincode, 1),
                )
                .unwrap();
        }
    }

    #[test]
    fn replay_determinism_identical_hashes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.astra_jl");
        let events = vec![
            make_test_event(1, 1_000_000, 2_934_567_000_000),
            make_test_event(2, 2_000_000, 2_935_000_000_000),
            make_test_event(3, 3_000_000, 2_933_000_000_000),
        ];
        write_events_to_journal(&path, &events);

        let report1 = StreamReplayEngine::replay_journal(&path).unwrap();
        let report2 = StreamReplayEngine::replay_journal(&path).unwrap();

        assert_eq!(report1.events_replayed, 3);
        assert_eq!(report2.events_replayed, 3);
        assert_eq!(report1.final_hash, report2.final_hash);
        assert!(report1.violations.is_empty());
    }

    #[test]
    fn replay_empty_journal() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.astra_jl");
        write_events_to_journal(&path, &[]);

        let report = StreamReplayEngine::replay_journal(&path).unwrap();
        assert_eq!(report.events_replayed, 0);
        assert_eq!(report.final_hash, [0u8; 32]);
    }

    #[test]
    fn detect_duplicate_trade_id() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.astra_jl");
        let events = vec![
            make_test_event(1, 1_000_000, 100_000_000),
            make_test_event(1, 2_000_000, 200_000_000),
        ];
        write_events_to_journal(&path, &events);

        let report = StreamReplayEngine::replay_journal(&path).unwrap();
        assert_eq!(report.events_replayed, 2);
        assert!(report
            .violations
            .iter()
            .any(|v| matches!(v, IntegrityViolation::DuplicateTradeId { trade_id: 1, .. })));
    }

    #[test]
    fn detect_out_of_order_timestamp() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.astra_jl");
        let events = vec![
            make_test_event(1, 2_000_000, 100_000_000),
            make_test_event(2, 1_000_000, 200_000_000),
        ];
        write_events_to_journal(&path, &events);

        let report = StreamReplayEngine::replay_journal(&path).unwrap();
        assert!(report.violations.iter().any(|v| matches!(
            v,
            IntegrityViolation::OutOfOrderTimestamp {
                prev_timestamp_us: 2_000_000,
                curr_timestamp_us: 1_000_000,
                ..
            }
        )));
    }

    #[test]
    fn detect_reconnect_gap() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.astra_jl");
        // trade_id jumps from 1 to 10 — 8 trades were missed
        let events = vec![
            make_test_event(1, 1_000_000, 100_000_000),
            make_test_event(10, 2_000_000, 100_000_000),
        ];
        write_events_to_journal(&path, &events);

        let report = StreamReplayEngine::replay_journal(&path).unwrap();
        assert!(report.violations.iter().any(|v| matches!(
            v,
            IntegrityViolation::ReconnectGap {
                prev_trade_id: 1,
                curr_trade_id: 10,
                ..
            }
        )));
    }

    #[test]
    fn detect_stream_discontinuity() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.astra_jl");
        // 90-second gap between timestamps exceeds 60-second threshold
        let events = vec![
            make_test_event(1, 1_000_000, 100_000_000),
            make_test_event(2, 91_000_000, 100_000_000), // 90s gap
        ];
        write_events_to_journal(&path, &events);

        let report = StreamReplayEngine::replay_journal(&path).unwrap();
        assert!(report.violations.iter().any(|v| matches!(
            v,
            IntegrityViolation::StreamDiscontinuity { gap_us, .. } if *gap_us == 90_000_000
        )));
    }

    #[test]
    fn replay_directory_deterministic_order() {
        let dir = tempfile::tempdir().unwrap();

        // Create files with names that sort lexicographically
        // "btcusdt_2024_01_15_09.astra_jl" < "btcusdt_2024_01_15_10.astra_jl"
        let file_a = dir.path().join("btcusdt_2024_01_15_09.astra_jl");
        let file_b = dir.path().join("btcusdt_2024_01_15_10.astra_jl");

        write_events_to_journal(&file_a, &[make_test_event(1, 1_000_000, 100_000_000)]);
        write_events_to_journal(&file_b, &[make_test_event(2, 2_000_000, 200_000_000)]);

        let report1 = StreamReplayEngine::replay_directory(dir.path()).unwrap();
        let report2 = StreamReplayEngine::replay_directory(dir.path()).unwrap();

        // Must be deterministic
        assert_eq!(report1.final_hash, report2.final_hash);
        assert_eq!(report1.events_replayed, 2);
        // Files must be replayed in lexicographic order
        assert_eq!(
            report1.files_replayed[0].file_name().unwrap(),
            "btcusdt_2024_01_15_09.astra_jl"
        );
        assert_eq!(
            report1.files_replayed[1].file_name().unwrap(),
            "btcusdt_2024_01_15_10.astra_jl"
        );
    }

    #[test]
    fn replay_directory_hash_differs_from_reversed_order() {
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();

        // dir1: file_a then file_b (lex order)
        let a1 = dir1.path().join("aaa.astra_jl");
        let b1 = dir1.path().join("bbb.astra_jl");
        write_events_to_journal(&a1, &[make_test_event(1, 1_000_000, 100_000_000)]);
        write_events_to_journal(&b1, &[make_test_event(2, 2_000_000, 200_000_000)]);

        // dir2: same events but reversed order in separate files
        let a2 = dir2.path().join("aaa.astra_jl");
        let b2 = dir2.path().join("bbb.astra_jl");
        write_events_to_journal(&a2, &[make_test_event(2, 2_000_000, 200_000_000)]);
        write_events_to_journal(&b2, &[make_test_event(1, 1_000_000, 100_000_000)]);

        let r1 = StreamReplayEngine::replay_directory(dir1.path()).unwrap();
        let r2 = StreamReplayEngine::replay_directory(dir2.path()).unwrap();

        // Different ordering → different hash
        assert_ne!(r1.final_hash, r2.final_hash);
    }

    #[test]
    fn replay_throughput_computable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bench.astra_jl");
        let events: Vec<_> = (0..100)
            .map(|i| make_test_event(i, i * 1000, 100_000_000))
            .collect();
        write_events_to_journal(&path, &events);

        let report = StreamReplayEngine::replay_journal(&path).unwrap();
        assert_eq!(report.events_replayed, 100);
        let _eps = report.events_per_sec();
    }

    #[test]
    fn different_events_produce_different_hashes() {
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();

        let p1 = dir1.path().join("a.astra_jl");
        let p2 = dir2.path().join("a.astra_jl");
        write_events_to_journal(&p1, &[make_test_event(1, 1_000_000, 100_000_000)]);
        write_events_to_journal(&p2, &[make_test_event(1, 1_000_000, 200_000_000)]);

        let r1 = StreamReplayEngine::replay_journal(&p1).unwrap();
        let r2 = StreamReplayEngine::replay_journal(&p2).unwrap();
        assert_ne!(r1.final_hash, r2.final_hash);
    }

    #[test]
    fn csv_export_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.astra_jl");
        write_events_to_journal(&path, &[make_test_event(1, 1_000_000, 100_000_000)]);

        let report = StreamReplayEngine::replay_journal(&path).unwrap();
        let csv_path = dir.path().join("bench.csv");
        export_csv(&report, &csv_path).unwrap();

        let content = std::fs::read_to_string(&csv_path).unwrap();
        assert!(content.contains("events_replayed"));
        assert!(content.contains("1,")); // 1 event replayed
    }
}
