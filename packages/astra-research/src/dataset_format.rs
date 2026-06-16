//! .astra_ds — AstraQuant Crisis Dataset File Format
//!
//! Binary layout:
//!   [MAGIC: 8 bytes "ASTRA_DS"]
//!   [VERSION: u32 le]
//!   [HEADER: length-prefixed canonical bincode]
//!     { version, crisis_name, date_range, event_count }
//!   [BODY: all AstraEvents, each length-prefixed u64 le]
//!   [FOOTER: Blake3 hash of entire BODY segment, 32 bytes]
//!
//! The footer is computed over raw BODY bytes only (not the header),
//! allowing streaming verification of body integrity independently.

use astra_core::events::AstraEvent;
use astra_core::hashing::hash_bytes;
use astra_core::serialization::{deserialize_canonical, serialize_canonical};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

const MAGIC: &[u8; 8] = b"ASTRA_DS";
const FORMAT_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatasetHeader {
    pub format_version: u32,
    pub crisis_name: String,
    pub date_range: String,
    pub event_count: u64,
}

// ---------------------------------------------------------------------------
// Loaded dataset
// ---------------------------------------------------------------------------

pub struct CrisisDataset {
    pub header: DatasetHeader,
    pub events: Vec<AstraEvent>,
}

// ---------------------------------------------------------------------------
// Writer
// ---------------------------------------------------------------------------

pub struct DatasetWriter;

impl DatasetWriter {
    /// Serialize a set of AstraEvents into the .astra_ds format.
    pub fn write(
        path: &Path,
        crisis_name: &str,
        date_range: &str,
        events: &[AstraEvent],
    ) -> io::Result<()> {
        let header = DatasetHeader {
            format_version: FORMAT_VERSION,
            crisis_name: crisis_name.to_string(),
            date_range: date_range.to_string(),
            event_count: events.len() as u64,
        };

        let header_bytes =
            serialize_canonical(&header).map_err(|e| io::Error::other(e.to_string()))?;

        // Serialize all events into the body buffer
        let mut body_buf: Vec<u8> = Vec::new();
        for event in events {
            let event_bytes =
                serialize_canonical(event).map_err(|e| io::Error::other(e.to_string()))?;
            let len = event_bytes.len() as u64;
            body_buf.extend_from_slice(&len.to_le_bytes());
            body_buf.extend_from_slice(&event_bytes);
        }

        let footer = hash_bytes(&body_buf);

        let mut file = File::create(path)?;
        // Magic
        file.write_all(MAGIC)?;
        // Format version
        file.write_all(&FORMAT_VERSION.to_le_bytes())?;
        // Header (length-prefixed)
        file.write_all(&(header_bytes.len() as u64).to_le_bytes())?;
        file.write_all(&header_bytes)?;
        // Body
        file.write_all(&body_buf)?;
        // Footer (Blake3 of body)
        file.write_all(&footer)?;
        file.flush()?;

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Reader
// ---------------------------------------------------------------------------

pub struct DatasetReader;

impl DatasetReader {
    /// Parse and integrity-verify a .astra_ds file.
    pub fn read(path: &Path) -> io::Result<CrisisDataset> {
        let mut file = File::open(path)?;
        let mut raw: Vec<u8> = Vec::new();
        file.read_to_end(&mut raw)?;

        let mut cursor = 0usize;

        // Validate magic
        if raw.len() < 8 || &raw[..8] != MAGIC {
            return Err(io::Error::other("Invalid .astra_ds magic header"));
        }
        cursor += 8;

        // Format version
        let ver = u32::from_le_bytes(raw[cursor..cursor + 4].try_into().unwrap());
        cursor += 4;
        if ver != FORMAT_VERSION {
            return Err(io::Error::other(format!(
                "Unsupported dataset version: {ver}"
            )));
        }

        // Header length
        let hdr_len = u64::from_le_bytes(raw[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let header: DatasetHeader = deserialize_canonical(&raw[cursor..cursor + hdr_len])
            .map_err(|e| io::Error::other(e.to_string()))?;
        cursor += hdr_len;

        // Body + Footer: footer is last 32 bytes
        if raw.len() < cursor + 32 {
            return Err(io::Error::other("Dataset truncated — footer missing"));
        }
        let footer_start = raw.len() - 32;
        let body_bytes = &raw[cursor..footer_start];
        let stored_footer: [u8; 32] = raw[footer_start..].try_into().unwrap();
        let computed_footer = hash_bytes(body_bytes);

        if computed_footer != stored_footer {
            return Err(io::Error::other(
                "INTEGRITY VIOLATION: Blake3 footer mismatch — dataset corrupted",
            ));
        }

        // Decode events from body
        let mut events: Vec<AstraEvent> = Vec::with_capacity(header.event_count as usize);
        let mut body_cursor = 0usize;
        while body_cursor < body_bytes.len() {
            if body_cursor + 8 > body_bytes.len() {
                return Err(io::Error::other("Body truncated mid-event length prefix"));
            }
            let evt_len =
                u64::from_le_bytes(body_bytes[body_cursor..body_cursor + 8].try_into().unwrap())
                    as usize;
            body_cursor += 8;

            if body_cursor + evt_len > body_bytes.len() {
                return Err(io::Error::other("Body truncated mid-event payload"));
            }
            let event: AstraEvent =
                deserialize_canonical(&body_bytes[body_cursor..body_cursor + evt_len])
                    .map_err(|e| io::Error::other(e.to_string()))?;
            events.push(event);
            body_cursor += evt_len;
        }

        Ok(CrisisDataset { header, events })
    }
}
