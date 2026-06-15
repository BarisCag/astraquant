use crate::venue::VenueId;
use astra_core::types::{Price, Quantity};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VenueQuote {
    pub venue_id: VenueId,
    pub price: Price,
    pub quantity: Quantity,
}

#[derive(Clone, Debug, Default)]
pub struct ConsolidatedBbo {
    pub best_bid: Option<VenueQuote>,
    pub best_ask: Option<VenueQuote>,
}

#[derive(Clone, Debug, Default)]
pub struct ConsolidatedDepth {
    // Price -> VenueId -> Quantity
    pub bids: BTreeMap<Price, BTreeMap<VenueId, Quantity>>,
    pub asks: BTreeMap<Price, BTreeMap<VenueId, Quantity>>,
}

impl ConsolidatedDepth {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn get_bbo(&self) -> ConsolidatedBbo {
        let best_bid = self.bids.iter().next_back().map(|(price, venues)| {
            let (venue_id, quantity) = venues.iter().next().unwrap();
            VenueQuote {
                venue_id: *venue_id,
                price: *price,
                quantity: *quantity,
            }
        });

        let best_ask = self.asks.iter().next().map(|(price, venues)| {
            let (venue_id, quantity) = venues.iter().next().unwrap();
            VenueQuote {
                venue_id: *venue_id,
                price: *price,
                quantity: *quantity,
            }
        });

        ConsolidatedBbo { best_bid, best_ask }
    }
}
