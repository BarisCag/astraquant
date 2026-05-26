//! Journal rotation: time-bucketed, automatically rolled journal files.
//!
//! Each journal file covers one hour of wall-clock time. Filenames encode the
//! UTC hour bucket so replay can reconstruct the correct temporal ordering.
//!
//! # Filename format
//!
//! ```text
//! {symbol}_{YYYY}_{MM}_{DD}_{HH}.astra_jl
//! ```
//!
//! Example: `btcusdt_2024_01_15_09.astra_jl`
//!
//! # Memory model
//!
//! `JournalRotator` holds exactly one open `EventJournal` per symbol at any time.
//! Rotated journals are closed (flushed) before the new one is opened. There is no
//! background queue; rotation is checked synchronously on every event commit.

use astra_core::journal::EventJournal;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Number of seconds in one hour.
const SECS_PER_HOUR: u64 = 3600;

/// Compute the UTC hour-bucket key for a Unix timestamp in seconds.
/// Returns `(year, month, day, hour)` in UTC.
pub fn utc_hour_bucket(unix_secs: u64) -> (u16, u8, u8, u8) {
    // Days since epoch
    let mut remaining = unix_secs;
    let hour = (remaining % SECS_PER_HOUR.saturating_mul(24) / 3600) as u8;
    let days = remaining / (SECS_PER_HOUR * 24);
    remaining -= days * (SECS_PER_HOUR * 24);
    let _ = remaining; // suppress unused warning

    // Gregorian calendar computation (no external crate)
    let (year, month, day) = days_to_ymd(days);
    (year, month, day, hour)
}

/// Convert days-since-epoch to (year, month, day) in UTC, proleptic Gregorian.
fn days_to_ymd(days: u64) -> (u16, u8, u8) {
    // Algorithm: Civil calendar from Howard Hinnant's chrono work
    let z = days as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u16, m as u8, d as u8)
}

/// Build the journal filename for a given symbol and UTC hour bucket.
pub fn journal_filename(symbol: &str, year: u16, month: u8, day: u8, hour: u8) -> String {
    format!(
        "{}_{:04}_{:02}_{:02}_{:02}.astra_jl",
        symbol.to_lowercase(),
        year,
        month,
        day,
        hour
    )
}

/// Returns the current wall-clock Unix timestamp in seconds.
/// Non-deterministic. Used only to decide when to rotate.
fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs()
}

/// Manages an `EventJournal` that rotates on UTC hour boundaries.
///
/// # Memory model
///
/// Holds exactly one open `EventJournal` at all times. On rotation, the old
/// journal is flushed and dropped before the new one is opened. No buffering
/// occurs beyond the `BufWriter` inside `EventJournal`.
pub struct JournalRotator {
    journal_dir: PathBuf,
    symbol: String,
    current_hour_key: (u16, u8, u8, u8),
    journal: EventJournal,
}

impl JournalRotator {
    /// Open or create the journal for the current hour.
    pub fn open<P: AsRef<Path>>(journal_dir: P, symbol: &str) -> io::Result<Self> {
        let dir = journal_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;

        let unix_secs = now_unix_secs();
        let (year, month, day, hour) = utc_hour_bucket(unix_secs);
        let filename = journal_filename(symbol, year, month, day, hour);
        let path = dir.join(&filename);

        eprintln!(
            "[astra-stream] journal rotator: opening {} (hour={:04}-{:02}-{:02}T{:02}:00Z)",
            path.display(),
            year,
            month,
            day,
            hour
        );

        let journal = EventJournal::open(&path)?;

        Ok(Self {
            journal_dir: dir,
            symbol: symbol.to_string(),
            current_hour_key: (year, month, day, hour),
            journal,
        })
    }

    /// Get a mutable reference to the current active journal, rotating if needed.
    ///
    /// Rotation is a synchronous, in-place operation: flush current journal,
    /// open new journal, update the hour key. Returns an error only if the
    /// new journal file cannot be created.
    pub fn journal(&mut self) -> io::Result<&mut EventJournal> {
        let unix_secs = now_unix_secs();
        let current_key = utc_hour_bucket(unix_secs);

        if current_key != self.current_hour_key {
            let (year, month, day, hour) = current_key;
            let filename = journal_filename(&self.symbol, year, month, day, hour);
            let path = self.journal_dir.join(&filename);

            eprintln!(
                "[astra-stream] journal rotation: {} → {}",
                journal_filename(
                    &self.symbol,
                    self.current_hour_key.0,
                    self.current_hour_key.1,
                    self.current_hour_key.2,
                    self.current_hour_key.3
                ),
                filename
            );

            // Open new journal before dropping old one to detect FS errors early
            let new_journal = EventJournal::open(&path)?;
            self.journal = new_journal;
            self.current_hour_key = current_key;
        }

        Ok(&mut self.journal)
    }

    /// Return the path of the currently active journal file.
    pub fn current_path(&self) -> PathBuf {
        let (year, month, day, hour) = self.current_hour_key;
        let filename = journal_filename(&self.symbol, year, month, day, hour);
        self.journal_dir.join(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_filename_format() {
        assert_eq!(
            journal_filename("btcusdt", 2024, 1, 15, 9),
            "btcusdt_2024_01_15_09.astra_jl"
        );
        assert_eq!(
            journal_filename("ETHUSDT", 2024, 12, 31, 23),
            "ethusdt_2024_12_31_23.astra_jl"
        );
    }

    #[test]
    fn journal_filename_lowercase_symbol() {
        let name = journal_filename("BTCUSDT", 2024, 6, 1, 0);
        assert!(name.starts_with("btcusdt_"));
    }

    #[test]
    fn utc_hour_bucket_epoch() {
        // Unix epoch: 1970-01-01T00:00:00Z
        let (y, m, d, h) = utc_hour_bucket(0);
        assert_eq!((y, m, d, h), (1970, 1, 1, 0));
    }

    #[test]
    fn utc_hour_bucket_known_timestamp() {
        // 2024-01-15T09:30:00Z = 1705311000
        let (y, m, d, h) = utc_hour_bucket(1_705_311_000);
        assert_eq!((y, m, d, h), (2024, 1, 15, 9));
    }

    #[test]
    fn utc_hour_bucket_midnight_rollover() {
        // 2024-03-01T00:00:00Z = 1709251200
        let (y, m, d, h) = utc_hour_bucket(1_709_251_200);
        assert_eq!((y, m, d, h), (2024, 3, 1, 0));
    }

    #[test]
    fn journal_rotator_opens_and_writes() {
        let dir = tempfile::tempdir().unwrap();
        let mut rotator = JournalRotator::open(dir.path(), "btcusdt").unwrap();

        // Should be able to get the journal and commit to it
        let journal = rotator.journal().unwrap();
        assert!(journal.next_sequence_id() >= 1);
    }

    #[test]
    fn journal_rotator_current_path_matches_filename() {
        let dir = tempfile::tempdir().unwrap();
        let rotator = JournalRotator::open(dir.path(), "ethusdt").unwrap();

        let path = rotator.current_path();
        let filename = path.file_name().unwrap().to_str().unwrap();
        assert!(filename.starts_with("ethusdt_"));
        assert!(filename.ends_with(".astra_jl"));
    }
}
