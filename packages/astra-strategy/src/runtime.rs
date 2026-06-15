use crate::agent::{AgentContext, StrategyAgent};
use crate::types::{MarketEvent, StrategyAction};
use astra_core::types::Quantity;
use std::collections::BTreeMap;

pub struct StrategyRuntime {
    pub agents: BTreeMap<u64, Box<dyn StrategyAgent>>,
    pub contexts: BTreeMap<u64, AgentContext>,
}

impl StrategyRuntime {
    pub fn new() -> Self {
        Self {
            agents: BTreeMap::new(),
            contexts: BTreeMap::new(),
        }
    }

    pub fn register_agent(&mut self, trader_id: u64, agent: Box<dyn StrategyAgent>) {
        self.agents.insert(trader_id, agent);
        self.contexts.insert(trader_id, AgentContext::default());
    }

    pub fn dispatch_market_event(&mut self, event: &MarketEvent) -> Vec<(u64, StrategyAction)> {
        let mut all_actions = Vec::new();

        for (trader_id, agent) in self.agents.iter_mut() {
            if let Some(ctx) = self.contexts.get_mut(trader_id) {
                match event {
                    MarketEvent::BookUpdate {
                        engine_sequence_id, ..
                    } => ctx.engine_sequence_id = *engine_sequence_id,
                    MarketEvent::TradeExecution {
                        engine_sequence_id, ..
                    } => ctx.engine_sequence_id = *engine_sequence_id,
                }

                let actions = agent.on_market_event(event, ctx);
                for act in actions {
                    all_actions.push((*trader_id, act));
                }
            }
        }

        all_actions
    }

    pub fn dispatch_fill(
        &mut self,
        trader_id: u64,
        order_id: u64,
        quantity: Quantity,
        price: i64,
    ) -> Vec<(u64, StrategyAction)> {
        let mut all_actions = Vec::new();
        if let (Some(agent), Some(ctx)) = (
            self.agents.get_mut(&trader_id),
            self.contexts.get_mut(&trader_id),
        ) {
            let actions = agent.on_fill(order_id, quantity, price, ctx);
            for act in actions {
                all_actions.push((trader_id, act));
            }
        }
        all_actions
    }

    pub fn dispatch_risk_violation(
        &mut self,
        trader_id: u64,
        reason: &str,
    ) -> Vec<(u64, StrategyAction)> {
        let mut all_actions = Vec::new();
        if let (Some(agent), Some(ctx)) = (
            self.agents.get_mut(&trader_id),
            self.contexts.get_mut(&trader_id),
        ) {
            let actions = agent.on_risk_violation(reason, ctx);
            for act in actions {
                all_actions.push((trader_id, act));
            }
        }
        all_actions
    }

    pub fn state_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        for (trader_id, agent) in &self.agents {
            hasher.update(&trader_id.to_le_bytes());
            hasher.update(&agent.state_hash());
        }
        for (trader_id, ctx) in &self.contexts {
            hasher.update(&trader_id.to_le_bytes());
            hasher.update(&ctx.inventory.to_le_bytes());
            hasher.update(&ctx.engine_sequence_id.to_le_bytes());
        }
        *hasher.finalize().as_bytes()
    }
}
