use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SettlementStatus {
    Pending,
    Settling,
    Settled,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetSettlementBucket {
    pub trader_id: u64,
    pub symbol: String,
    pub settlement_sequence: u64,
    pub net_quantity: i64,
    pub net_cash_movement: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettlementInstruction {
    pub instruction_id: u64,
    pub trader_id: u64,
    pub symbol: String,
    pub quantity: u64,
    pub price: u64,
    pub is_buy: bool,
    pub target_sequence: u64,
    pub status: SettlementStatus,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SettlementEngine {
    pub queues: BTreeMap<u64, Vec<SettlementInstruction>>,
    pub next_instruction_id: u64,
    pub default_delay_sequences: u64,
    pub holiday_sequences_remaining: u64,
}

impl SettlementEngine {
    pub fn new(default_delay_sequences: u64) -> Self {
        Self {
            queues: BTreeMap::new(),
            next_instruction_id: 1,
            default_delay_sequences,
            holiday_sequences_remaining: 0,
        }
    }

    pub fn activate_holiday(&mut self, sequences: u64) {
        self.holiday_sequences_remaining = sequences;
    }

    pub fn queue_trade(
        &mut self,
        trader_id: u64,
        symbol: String,
        quantity: u64,
        price: u64,
        is_buy: bool,
        current_sequence: u64,
    ) {
        let target_sequence =
            current_sequence + self.default_delay_sequences + self.holiday_sequences_remaining;
        let instruction = SettlementInstruction {
            instruction_id: self.next_instruction_id,
            trader_id,
            symbol,
            quantity,
            price,
            is_buy,
            target_sequence,
            status: SettlementStatus::Pending,
        };
        self.next_instruction_id += 1;
        self.queues
            .entry(target_sequence)
            .or_default()
            .push(instruction);
    }

    pub fn mature_obligations(&mut self, current_sequence: u64) -> Vec<NetSettlementBucket> {
        let mut buckets: BTreeMap<(u64, String), NetSettlementBucket> = BTreeMap::new();

        // Extract instructions up to current_sequence
        let mut mature = Vec::new();
        let sequences: Vec<u64> = self
            .queues
            .keys()
            .copied()
            .filter(|&seq| seq <= current_sequence)
            .collect();
        for seq in sequences {
            if let Some(mut insts) = self.queues.remove(&seq) {
                mature.append(&mut insts);
            }
        }

        for inst in mature {
            let key = (inst.trader_id, inst.symbol.clone());
            let bucket = buckets.entry(key).or_insert(NetSettlementBucket {
                trader_id: inst.trader_id,
                symbol: inst.symbol.clone(),
                settlement_sequence: current_sequence,
                net_quantity: 0,
                net_cash_movement: 0,
            });

            let notional = inst.quantity as i64 * inst.price as i64;
            if inst.is_buy {
                bucket.net_quantity += inst.quantity as i64;
                bucket.net_cash_movement -= notional;
            } else {
                bucket.net_quantity -= inst.quantity as i64;
                bucket.net_cash_movement += notional;
            }
        }

        buckets.into_values().collect()
    }
}
