pub mod market;
pub mod portfolio;
pub mod treasury;
pub mod risk;
pub mod alm;
pub mod admin;
pub mod ws;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ApiResponse<T> {
    pub data: T,
    pub timestamp_ns: u64,
    pub mode: String,
}

