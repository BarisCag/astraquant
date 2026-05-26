//! Deterministic event journaling subsystem.
//!
//! The `EventJournal` is the cryptographic source of truth for AstraQuant OS.
//! It enforces strictly monotonic sequence ordering and immutable append-only semantics.
//! There is no way to update or delete a historical event; state is purely a derivative
//! of this event log.

use crate::events::AstraEvent;
use crate::serialization::{deserialize_canonical, serialize_canonical};
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

pub struct EventJournal {
    file: BufWriter<File>,
    sequence_id: u64,
    event_count: u64,
    pub path: PathBuf,
}

impl EventJournal {
    /// Open an existing journal file for appending.
    /// Scans the file to determine current sequence position and event count.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let p = path.as_ref().to_path_buf();

        // Scan existing events to find the last sequence_id and count
        let (count, last_seq) = Self::scan_journal(&p)?;

        let file = OpenOptions::new().create(true).append(true).open(&p)?;
        let writer = BufWriter::new(file);
        Ok(Self {
            file: writer,
            sequence_id: last_seq + 1,
            event_count: count,
            path: p,
        })
    }

    /// Create a new journal file, truncating any existing content.
    pub fn create<P: AsRef<Path>>(path: P, _epoch: u64) -> io::Result<Self> {
        let p = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&p)?;
        let writer = BufWriter::new(file);
        Ok(Self {
            file: writer,
            sequence_id: 1,
            event_count: 0,
            path: p,
        })
    }

    /// Returns the number of events currently in the journal.
    pub fn len(&self) -> u64 {
        self.event_count
    }

    pub fn is_empty(&self) -> bool {
        self.event_count == 0
    }

    /// Returns the next sequence_id that will be assigned.
    pub fn next_sequence_id(&self) -> u64 {
        self.sequence_id
    }

    /// Commit a new event with auto-assigned sequence_id.
    pub fn commit(
        &mut self,
        timestamp_ns: u64,
        event_type: crate::events::EventType,
        payload: Vec<u8>,
        payload_metadata: crate::events::PayloadMetadata,
    ) -> io::Result<AstraEvent> {
        let event = AstraEvent::new(
            timestamp_ns,
            0, // will be assigned by append_internal
            event_type,
            payload,
            payload_metadata,
        );
        self.append_internal(event)
    }

    /// Append a pre-constructed event. Enforces strict monotonic sequence ordering.
    /// The event's sequence_id MUST match the journal's expected next_sequence_id.
    pub fn append(&mut self, event: &AstraEvent) -> io::Result<u64> {
        if event.sequence_id != self.sequence_id {
            return Err(io::Error::other(format!(
                "Sequence violation: expected {}, got {}",
                self.sequence_id, event.sequence_id
            )));
        }
        self.write_event(event)?;
        let seq = self.sequence_id;
        self.sequence_id += 1;
        self.event_count += 1;
        Ok(seq)
    }

    /// Internal append that auto-assigns sequence_id and returns the full event.
    fn append_internal(&mut self, mut event: AstraEvent) -> io::Result<AstraEvent> {
        event.sequence_id = self.sequence_id;
        self.write_event(&event)?;
        self.sequence_id += 1;
        self.event_count += 1;
        Ok(event)
    }

    /// Write a serialized event to the journal file.
    fn write_event(&mut self, event: &AstraEvent) -> io::Result<()> {
        let bytes = serialize_canonical(event).map_err(|e| io::Error::other(format!("{:?}", e)))?;
        let len = bytes.len() as u32;
        self.file.write_all(&len.to_le_bytes())?;
        self.file.write_all(&bytes)?;
        self.file.flush()
    }

    /// Iterate all events in the journal from the beginning.
    pub fn iter(&self) -> io::Result<impl Iterator<Item = io::Result<AstraEvent>>> {
        Self::iter_path(self.path.clone())
    }

    /// Iterate events starting after a given sequence_id.
    /// Returns events with sequence_id > after_seq.
    pub fn iter_from(
        &self,
        after_seq: u64,
    ) -> io::Result<impl Iterator<Item = io::Result<AstraEvent>>> {
        let iter = Self::iter_path(self.path.clone())?;
        Ok(SkipToIterator {
            inner: iter,
            after_seq,
        })
    }

    /// Static method to iterate a journal file by path.
    pub fn iter_path<P: AsRef<Path>>(
        path: P,
    ) -> io::Result<impl Iterator<Item = io::Result<AstraEvent>>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(JournalIterator { reader })
    }

    /// Scan a journal file to find event count and last sequence_id.
    fn scan_journal(path: &Path) -> io::Result<(u64, u64)> {
        if !path.exists() {
            return Ok((0, 0));
        }
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok((0, 0)),
            Err(e) => return Err(e),
        };

        let metadata = file.metadata()?;
        if metadata.len() == 0 {
            return Ok((0, 0));
        }

        let mut count = 0u64;
        let mut last_seq = 0u64;
        for event_result in (JournalIterator {
            reader: BufReader::new(file),
        }) {
            let event = event_result?;
            last_seq = event.sequence_id;
            count += 1;
        }
        Ok((count, last_seq))
    }
}

pub struct JournalIterator {
    reader: BufReader<File>,
}

impl Iterator for JournalIterator {
    type Item = io::Result<AstraEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut len_bytes = [0u8; 4];
        if let Err(e) = self.reader.read_exact(&mut len_bytes) {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                return None;
            }
            return Some(Err(e));
        }
        let len = u32::from_le_bytes(len_bytes) as usize;
        let mut buf = vec![0u8; len];
        if let Err(e) = self.reader.read_exact(&mut buf) {
            return Some(Err(e));
        }

        match deserialize_canonical(&buf) {
            Ok(event) => Some(Ok(event)),
            Err(e) => Some(Err(io::Error::other(format!("{:?}", e)))),
        }
    }
}

/// Iterator adapter that skips events with sequence_id <= after_seq.
struct SkipToIterator<I: Iterator<Item = io::Result<AstraEvent>>> {
    inner: I,
    after_seq: u64,
}

impl<I: Iterator<Item = io::Result<AstraEvent>>> Iterator for SkipToIterator<I> {
    type Item = io::Result<AstraEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some(Ok(event)) => {
                    if event.sequence_id > self.after_seq {
                        return Some(Ok(event));
                    }
                    // Skip events at or before after_seq
                }
                other => return other,
            }
        }
    }
}
