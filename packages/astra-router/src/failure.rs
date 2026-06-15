use crate::venue::VenueId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum VenueFailureEvent {
    VenuePaused {
        venue_id: VenueId,
        sequence_id: u64,
    },
    VenueOffline {
        venue_id: VenueId,
        sequence_id: u64,
    },
    VenueRecovered {
        venue_id: VenueId,
        sequence_id: u64,
    },
    LiquidityCollapse {
        venue_id: VenueId,
        symbol: String,
        sequence_id: u64,
        fraction_to_remove: u8, // Example representation
    },
    SpreadExpansion {
        venue_id: VenueId,
        symbol: String,
        sequence_id: u64,
        tick_expansion: u64,
    },
}
