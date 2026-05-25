//! Snapshot manager for deterministic checkpoint creation and restoration.
//!
//! Snapshots are hash-verified reconstruction boundaries.
//! `Snapshot + Journal Replay = Identical State Hash` — that is the invariant.

use crate::events::SnapshotMetadata;
use crate::hashing::hash_bytes;
use crate::replay::EventReducer;
use crate::serialization::{deserialize_canonical, serialize_canonical, SerializationError};
use std::fmt;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

const SNAPSHOT_MAGIC: &[u8; 8] = b"ASTRA_SS";
const SNAPSHOT_VERSION: u32 = 1;

// =============================================================================
// SnapshotError
// =============================================================================

#[derive(Debug)]
pub enum SnapshotError {
    IoError(String),
    InvalidHeader,
    VersionMismatch { expected: u32, found: u32 },
    ChecksumMismatch,
    SerializationError(String),
    InvalidStateHashLength,
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "Snapshot I/O error: {}", e),
            Self::InvalidHeader => write!(f, "Invalid snapshot header"),
            Self::VersionMismatch { expected, found } => {
                write!(
                    f,
                    "Snapshot version mismatch: expected {}, found {}",
                    expected, found
                )
            }
            Self::ChecksumMismatch => write!(f, "Snapshot checksum mismatch — data corrupted"),
            Self::SerializationError(e) => write!(f, "Snapshot serialization error: {}", e),
            Self::InvalidStateHashLength => write!(f, "State hash must be exactly 32 bytes"),
        }
    }
}

impl std::error::Error for SnapshotError {}

impl From<io::Error> for SnapshotError {
    fn from(e: io::Error) -> Self {
        SnapshotError::IoError(e.to_string())
    }
}

impl From<SerializationError> for SnapshotError {
    fn from(e: SerializationError) -> Self {
        SnapshotError::SerializationError(e.to_string())
    }
}

// =============================================================================
// Snapshot
// =============================================================================

/// A restored snapshot containing metadata and raw state bytes.
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub metadata: SnapshotMetadata,
    pub state_data: Vec<u8>,
    pub created_ns: u64,
}

impl Snapshot {
    /// Deserialize snapshot state data into a concrete type.
    /// Uses the canonical bincode config for determinism.
    pub fn restore_state<T: serde::de::DeserializeOwned>(&self) -> Result<T, SnapshotError> {
        deserialize_canonical(&self.state_data)
            .map_err(|e| SnapshotError::SerializationError(e.to_string()))
    }
}

// =============================================================================
// SnapshotManager
// =============================================================================

/// Manages creation and restoration of hash-verified state snapshots.
pub struct SnapshotManager;

impl SnapshotManager {
    /// Capture a snapshot from a reducer's current state.
    ///
    /// Serializes the reducer state using canonical bincode, computes
    /// blake3 hash, and writes the snapshot file.
    pub fn capture<R: EventReducer + serde::Serialize>(
        reducer: &R,
        subsystem_id: &str,
        created_ns: u64,
        path: &Path,
    ) -> Result<SnapshotMetadata, SnapshotError> {
        let state_data = serialize_canonical(reducer)?;
        let state_hash = hash_bytes(&state_data);
        let last_seq = reducer.last_applied_sequence_id().unwrap_or(0);

        let mut file = File::create(path)?;

        // Header
        file.write_all(SNAPSHOT_MAGIC)?;
        file.write_all(&SNAPSHOT_VERSION.to_le_bytes())?;
        file.write_all(&created_ns.to_le_bytes())?;
        file.write_all(&last_seq.to_le_bytes())?;
        file.write_all(&state_hash)?;

        // Subsystem ID (length-prefixed)
        let id_bytes = subsystem_id.as_bytes();
        file.write_all(&(id_bytes.len() as u16).to_le_bytes())?;
        file.write_all(id_bytes)?;

        // State data (length-prefixed + trailing checksum)
        file.write_all(&(state_data.len() as u64).to_le_bytes())?;
        file.write_all(&state_data)?;
        file.write_all(&state_hash)?; // trailing checksum = same as header hash
        file.flush()?;

        Ok(SnapshotMetadata::from_hash(
            last_seq,
            state_hash,
            subsystem_id.to_string(),
        ))
    }

    /// Restore a snapshot from file. Validates checksum before returning.
    pub fn restore(path: &Path) -> Result<Snapshot, SnapshotError> {
        let mut file = File::open(path)?;

        // Read and validate magic
        let mut magic = [0u8; 8];
        file.read_exact(&mut magic)?;
        if &magic != SNAPSHOT_MAGIC {
            return Err(SnapshotError::InvalidHeader);
        }

        // Version
        let mut buf4 = [0u8; 4];
        file.read_exact(&mut buf4)?;
        let version = u32::from_le_bytes(buf4);
        if version != SNAPSHOT_VERSION {
            return Err(SnapshotError::VersionMismatch {
                expected: SNAPSHOT_VERSION,
                found: version,
            });
        }

        // Created timestamp
        let mut buf8 = [0u8; 8];
        file.read_exact(&mut buf8)?;
        let created_ns = u64::from_le_bytes(buf8);

        // Last sequence ID
        file.read_exact(&mut buf8)?;
        let last_sequence_id = u64::from_le_bytes(buf8);

        // State hash from header
        let mut header_hash = [0u8; 32];
        file.read_exact(&mut header_hash)?;

        // Subsystem ID
        let mut buf2 = [0u8; 2];
        file.read_exact(&mut buf2)?;
        let id_len = u16::from_le_bytes(buf2) as usize;
        let mut id_buf = vec![0u8; id_len];
        file.read_exact(&mut id_buf)?;
        let subsystem_id = String::from_utf8_lossy(&id_buf).to_string();

        // State data
        file.read_exact(&mut buf8)?;
        let data_len = u64::from_le_bytes(buf8) as usize;
        let mut state_data = vec![0u8; data_len];
        file.read_exact(&mut state_data)?;

        // Trailing checksum
        let mut trailing_checksum = [0u8; 32];
        file.read_exact(&mut trailing_checksum)?;

        // VALIDATE: compute hash of state_data, compare to both checksums
        let computed_hash = hash_bytes(&state_data);
        if computed_hash != header_hash || computed_hash != trailing_checksum {
            return Err(SnapshotError::ChecksumMismatch);
        }

        let metadata = SnapshotMetadata::from_hash(last_sequence_id, header_hash, subsystem_id);

        Ok(Snapshot {
            metadata,
            state_data,
            created_ns,
        })
    }

    /// Quick-verify a snapshot file's integrity without fully loading it.
    pub fn verify(path: &Path) -> Result<SnapshotMetadata, SnapshotError> {
        let snapshot = Self::restore(path)?;
        Ok(snapshot.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventType, PayloadMetadata};
    use crate::hashing::DeterministicState;
    use crate::journal::EventJournal;
    use crate::replay::EventReducer;
    use serde::{Deserialize, Serialize};
    use std::fs;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_artifacts");
        fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_file(path);
    }

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct TestReducer {
        count: u64,
        sum: u64,
        last_seq: Option<u64>,
    }

    impl TestReducer {
        fn new() -> Self {
            Self {
                count: 0,
                sum: 0,
                last_seq: None,
            }
        }
    }

    impl DeterministicState for TestReducer {
        fn state_hash(&self) -> [u8; 32] {
            let bytes = serialize_canonical(self).expect("serialization failed");
            hash_bytes(&bytes)
        }
    }

    impl EventReducer for TestReducer {
        type Error = String;
        fn apply(&mut self, event: &crate::events::AstraEvent) -> Result<(), String> {
            self.count += 1;
            self.sum += event.payload.iter().map(|b| *b as u64).sum::<u64>();
            self.last_seq = Some(event.sequence_id);
            Ok(())
        }
        fn last_applied_sequence_id(&self) -> Option<u64> {
            self.last_seq
        }
    }

    #[test]
    fn test_snapshot_capture_and_restore() {
        let snap_path = temp_path("test_snap.astra_ss");
        let jl_path = temp_path("test_snap_jl.astra_jl");
        cleanup(&snap_path);
        cleanup(&jl_path);

        let mut journal = EventJournal::create(&jl_path, 1_700_000_000_000_000_000).unwrap();
        let mut reducer = TestReducer::new();

        for i in 1..=5u64 {
            let event = journal
                .commit(
                    1_700_000_000_000_000_000 + i * 1_000_000,
                    EventType::MarketTick,
                    vec![i as u8],
                    PayloadMetadata::raw(),
                )
                .unwrap();
            reducer.apply(&event).unwrap();
        }

        let meta = SnapshotManager::capture(
            &reducer,
            "test-engine",
            1_700_000_000_000_000_000,
            &snap_path,
        )
        .unwrap();
        assert_eq!(meta.last_sequence_id, 5);
        assert_eq!(meta.state_hash, reducer.state_hash());

        let snapshot = SnapshotManager::restore(&snap_path).unwrap();
        let restored: TestReducer = snapshot.restore_state().unwrap();
        assert_eq!(restored, reducer);
        assert_eq!(restored.state_hash(), reducer.state_hash());

        cleanup(&snap_path);
        cleanup(&jl_path);
    }
}
