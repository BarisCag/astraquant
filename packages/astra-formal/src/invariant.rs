use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FormalInvariant {
    Determinism,
    Monotonicity,
    ReplayEquivalence,
    LineageContinuity,
    AggregationParity,
    ShardEquivalence,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvariantProof {
    pub invariant_id: String,
    pub is_valid: bool,
    pub violation_sequence: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeterminismInvariant {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SequenceMonotonicityInvariant {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayEquivalenceInvariant {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineageContinuityInvariant {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregationParityInvariant {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DistributedShardEquivalenceInvariant {}
