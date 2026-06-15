use serde::{Deserialize, Serialize};
use astra_core::events::AstraEvent;
use astra_core::marketdata::MarketTick;
use astra_core::serialization::deserialize_canonical;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateVector {
    pub features: Vec<f32>,
}

pub fn event_to_tensor(event: &AstraEvent, current_position: i64) -> StateVector {
    let mut features = vec![0.0; 4];
    
    // Feature 0: Timestamp (normalized roughly)
    features[0] = (event.timestamp_ns % 1_000_000_000) as f32 / 1_000_000_000.0;
    
    // Attempt to extract market tick data
    if let Ok(tick) = deserialize_canonical::<MarketTick>(&event.payload) {
        features[1] = tick.bid_price.0 as f32; // Price
        features[2] = tick.bid_quantity.0 as f32; // Volume
    }
    
    // Feature 3: Current Position
    features[3] = current_position as f32;
    
    StateVector { features }
}
