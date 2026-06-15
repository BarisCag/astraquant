use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountSolvencyState {
    Solvent,
    Insolvent,
    Defaulted,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FundingAccount {
    pub trader_id: u64,
    pub balance: i64,
    pub pending_credits: i64,
    pub pending_debits: i64,
    pub solvency_state: AccountSolvencyState,
}

impl FundingAccount {
    pub fn new(trader_id: u64, initial_balance: i64) -> Self {
        Self {
            trader_id,
            balance: initial_balance,
            pending_credits: 0,
            pending_debits: 0,
            solvency_state: AccountSolvencyState::Solvent,
        }
    }

    pub fn apply_funding(&mut self, amount: i64) {
        self.balance = self.balance.saturating_add(amount);
        self.update_solvency();
    }

    pub fn update_solvency(&mut self) {
        if self.balance < 0 {
            if self.solvency_state != AccountSolvencyState::Defaulted {
                self.solvency_state = AccountSolvencyState::Insolvent;
            }
        } else {
            self.solvency_state = AccountSolvencyState::Solvent;
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FundingLedger {
    pub accounts: BTreeMap<u64, FundingAccount>,
}

impl FundingLedger {
    pub fn new() -> Self {
        Self {
            accounts: BTreeMap::new(),
        }
    }

    pub fn ensure_account(&mut self, trader_id: u64) {
        self.accounts.entry(trader_id).or_insert_with(|| FundingAccount::new(trader_id, 0));
    }

    pub fn get_balance(&self, trader_id: u64) -> i64 {
        self.accounts.get(&trader_id).map(|a| a.balance).unwrap_or(0)
    }

    pub fn apply_cash_movement(&mut self, trader_id: u64, amount: i64) {
        let account = self.accounts.entry(trader_id).or_insert_with(|| FundingAccount::new(trader_id, 0));
        account.apply_funding(amount);
    }
}
