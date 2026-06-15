//! Phase 14A: Behavioral Agent Ecology
//!
//! Defines the 5 behavioral agents that execute inside the WasmSandbox.

use crate::agent::{AgentDecision, AgentId, AgentIntent};
use astra_core::events::BehavioralSeed;
use astra_core::sandbox::WasmSandbox;

pub struct BehavioralAgentEnvironment {
    pub sandbox: WasmSandbox,
    pub seed: BehavioralSeed,
    pub anchor_price: i64,
    pub prev_price: i64,
    pub current_price: i64,
    pub volatility_index: f64,
    pub price_history: Vec<i64>,
}

impl BehavioralAgentEnvironment {
    pub fn new(seed: BehavioralSeed, anchor_price: i64) -> Self {
        Self {
            sandbox: WasmSandbox::new(100_000), // Gas limit
            seed,
            anchor_price,
            prev_price: anchor_price,
            current_price: anchor_price,
            volatility_index: 0.0,
            price_history: Vec::new(),
        }
    }

    pub fn update_market(&mut self, price: i64) {
        self.prev_price = self.current_price;
        self.current_price = price;
        self.price_history.push(price);
        if self.price_history.len() > 10 {
            self.price_history.remove(0);
        }

        let diff = (self.current_price - self.prev_price).abs() as f64;
        self.volatility_index = (self.volatility_index * 0.9) + (diff * 0.1);
    }
}

// 1. HerdingAgent
pub fn evaluate_herding(env: &mut BehavioralAgentEnvironment) -> AgentDecision {
    let _ = env.sandbox.gas_meter.consume(500); // Enforce WASM gas
    let mut intents = Vec::new();

    if env.price_history.len() >= 3 {
        let recent_drop = env.prev_price - env.current_price;
        if recent_drop > 0 {
            // Price dropping, herd sells
            let strength = (recent_drop as f64 * env.seed.herding_factor).max(0.0);
            if strength > 5000.0 {
                // Threshold
                intents.push(AgentIntent {
                    agent_id: AgentId("Herding1".to_string()),
                    target_venue: 0,
                    symbol: "SPX".to_string(),
                    intent_type: "SELL".to_string(),
                    size: strength as u64,
                    price: env.current_price as u64,
                });
            }
        }
    }
    AgentDecision {
        intents,
        transition_reason: None,
    }
}

// 2. ProspectAgent (Loss Aversion)
pub fn evaluate_prospect(env: &mut BehavioralAgentEnvironment) -> AgentDecision {
    let _ = env.sandbox.gas_meter.consume(600); // Enforce WASM gas
    let mut intents = Vec::new();

    let change = env.current_price - env.anchor_price;
    if change < 0 {
        // Losses hurt more (λ loss_aversion)
        let perceived_loss = (change.abs() as f64) * env.seed.loss_aversion;
        if perceived_loss > 10_000.0 {
            intents.push(AgentIntent {
                agent_id: AgentId("Prospect1".to_string()),
                target_venue: 0,
                symbol: "SPX".to_string(),
                intent_type: "SELL".to_string(),
                size: (perceived_loss / 100.0) as u64,
                price: env.current_price as u64,
            });
        }
    }
    AgentDecision {
        intents,
        transition_reason: None,
    }
}

// 3. AnchorAgent
pub fn evaluate_anchor(env: &mut BehavioralAgentEnvironment) -> AgentDecision {
    let _ = env.sandbox.gas_meter.consume(400); // Enforce WASM gas
    let mut intents = Vec::new();

    let dist = (env.current_price - env.anchor_price).abs() as f64;
    if dist > 5000.0 {
        // Revert to anchor
        let pull = dist * env.seed.anchoring_bias;
        let side = if env.current_price > env.anchor_price {
            "SELL"
        } else {
            "BUY"
        };
        intents.push(AgentIntent {
            agent_id: AgentId("Anchor1".to_string()),
            target_venue: 0,
            symbol: "SPX".to_string(),
            intent_type: side.to_string(),
            size: pull as u64,
            price: env.current_price as u64,
        });
    }
    AgentDecision {
        intents,
        transition_reason: None,
    }
}

// 4. SalienceAgent
pub fn evaluate_salience(env: &mut BehavioralAgentEnvironment) -> AgentDecision {
    let _ = env.sandbox.gas_meter.consume(550); // Enforce WASM gas
    let mut intents = Vec::new();

    if env.volatility_index > 2000.0 {
        // Overreact to high volatility
        let reaction = env.volatility_index * env.seed.attention_salience;
        intents.push(AgentIntent {
            agent_id: AgentId("Salience1".to_string()),
            target_venue: 0,
            symbol: "SPX".to_string(),
            intent_type: "SELL".to_string(), // Salience in crashes usually triggers panic selling
            size: reaction as u64,
            price: env.current_price as u64,
        });
    }
    AgentDecision {
        intents,
        transition_reason: None,
    }
}

// 5. LiquidityWithdrawalAgent
pub fn evaluate_liquidity_withdrawal(env: &mut BehavioralAgentEnvironment) -> AgentDecision {
    let _ = env.sandbox.gas_meter.consume(700); // Enforce WASM gas
    let mut intents = Vec::new();

    if env.volatility_index > 3000.0 {
        // Withdraw liquidity during stress
        intents.push(AgentIntent {
            agent_id: AgentId("LiqWithdrawal1".to_string()),
            target_venue: 0,
            symbol: "SPX".to_string(),
            intent_type: "CANCEL_ALL".to_string(),
            size: 0,
            price: 0,
        });
    }
    AgentDecision {
        intents,
        transition_reason: None,
    }
}
