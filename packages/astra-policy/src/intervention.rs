use crate::policy::PolicyExecutionWindow;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LiquidityInjectionFacility {
    pub facility_id: u64,
    pub total_capacity: u64,
    pub injected_amount: u64,
    pub window: PolicyExecutionWindow,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EmergencyFundingWindow {
    pub window_id: u64,
    pub rate_discount_ppm: u64,
    pub window: PolicyExecutionWindow,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CollateralReliefProgram {
    pub program_id: u64,
    pub haircut_reduction_ppm: u64,
    pub window: PolicyExecutionWindow,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SettlementGuaranteeFacility {
    pub guarantee_id: u64,
    pub window: PolicyExecutionWindow,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CentralBankRepoWindow {
    pub repo_id: u64,
    pub target_rate_ppm: u64,
    pub window: PolicyExecutionWindow,
}

impl LiquidityInjectionFacility {
    pub fn is_active(&self, sequence: u64) -> bool {
        sequence >= self.window.start_sequence && sequence <= self.window.end_sequence
    }
}
