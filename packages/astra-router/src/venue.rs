use astra_lob::book::LimitOrderBook;
use bincode::Options;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VenueId(pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum VenueStatus {
    Active,
    Paused,
    RejectOnly,
    Offline,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VenueFeeModel {
    pub maker_fee_bps: i64,
    pub taker_fee_bps: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VenueLatencyProfile {
    pub ingress_delay_sequences: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VenueState {
    pub venue_id: VenueId,
    pub books: BTreeMap<String, LimitOrderBook>,
    pub fee_model: VenueFeeModel,
    pub latency_profile: VenueLatencyProfile,
    pub venue_sequence_id: u64,
    pub status: VenueStatus,
}

impl VenueState {
    pub fn new(
        venue_id: VenueId,
        fee_model: VenueFeeModel,
        latency_profile: VenueLatencyProfile,
    ) -> Self {
        Self {
            venue_id,
            books: BTreeMap::new(),
            fee_model,
            latency_profile,
            venue_sequence_id: 0,
            status: VenueStatus::Active,
        }
    }

    pub fn state_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&[self.venue_id.0]);
        hasher.update(&self.fee_model.maker_fee_bps.to_le_bytes());
        hasher.update(&self.fee_model.taker_fee_bps.to_le_bytes());
        hasher.update(&self.latency_profile.ingress_delay_sequences.to_le_bytes());
        hasher.update(&self.venue_sequence_id.to_le_bytes());
        hasher.update(&[self.status as u8]);
        
        for (symbol, book) in &self.books {
            hasher.update(symbol.as_bytes());
            let book_bytes = bincode::options()
                .with_little_endian()
                .with_fixint_encoding()
                .serialize(book)
                .expect("Book serialization failed");
            let mut h = blake3::Hasher::new();
            h.update(&book_bytes);
            hasher.update(h.finalize().as_bytes());
        }
        
        *hasher.finalize().as_bytes()
    }
}
