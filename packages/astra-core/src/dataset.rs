use crate::hashing::{hash_bytes, verify_hash_equality};
use crate::marketdata::MarketTick;
use crate::serialization::{deserialize_canonical, serialize_canonical};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatasetHeader {
    pub tick_count: u64,
    pub checksum: [u8; 32],
}

pub struct Dataset;

impl Dataset {
    pub fn save(path: &str, ticks: &[MarketTick]) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        let data_bytes = serialize_canonical(&ticks).unwrap();
        let checksum = hash_bytes(&data_bytes);

        let header = DatasetHeader {
            tick_count: ticks.len() as u64,
            checksum,
        };

        let header_bytes = serialize_canonical(&header).unwrap();
        file.write_all(&(header_bytes.len() as u64).to_le_bytes())?;
        file.write_all(&header_bytes)?;
        file.write_all(&data_bytes)?;

        Ok(())
    }

    pub fn load(path: &str) -> std::io::Result<Vec<MarketTick>> {
        let mut file = File::open(path)?;

        let mut len_buf = [0u8; 8];
        file.read_exact(&mut len_buf)?;
        let header_len = u64::from_le_bytes(len_buf);

        let mut header_bytes = vec![0u8; header_len as usize];
        file.read_exact(&mut header_bytes)?;
        let header: DatasetHeader = deserialize_canonical(&header_bytes).unwrap();

        let mut data_bytes = Vec::new();
        file.read_to_end(&mut data_bytes)?;

        let actual_checksum = hash_bytes(&data_bytes);
        if !verify_hash_equality(&header.checksum, &actual_checksum) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Checksum mismatch",
            ));
        }

        let ticks: Vec<MarketTick> = deserialize_canonical(&data_bytes).unwrap();
        Ok(ticks)
    }
}
