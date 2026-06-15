use astra_lob::types::LiquiditySide;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TimelineEvent {
    Accepted {
        execution_trace_id: u64,
        sequence: u64,
        trader_id: u64,
        symbol: String,
        side: String,
        quantity: u64,
        price: i64,
    },
    Fill {
        execution_trace_id: u64,
        sequence: u64,
        maker_trader_id: u64,
        taker_trader_id: u64,
        symbol: String,
        quantity: u64,
        price: i64,
        liquidity_side: LiquiditySide,
    },
    Update {
        execution_trace_id: u64,
        sequence: u64,
        symbol: String,
        queue_depth: usize,
    },
    Reject {
        execution_trace_id: u64,
        sequence: u64,
        trader_id: u64,
        reason: String,
    },
    Cancel {
        execution_trace_id: u64,
        sequence: u64,
        trader_id: u64,
        symbol: String,
    },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ReplayTimeline {
    pub events: Vec<TimelineEvent>,
    pub next_trace_id: u64,
}

impl ReplayTimeline {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            next_trace_id: 1,
        }
    }

    pub fn next_trace_id(&mut self) -> u64 {
        let id = self.next_trace_id;
        self.next_trace_id += 1;
        id
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_accepted(
        &mut self,
        execution_trace_id: u64,
        sequence: u64,
        trader_id: u64,
        symbol: String,
        side: String,
        quantity: u64,
        price: i64,
    ) {
        self.events.push(TimelineEvent::Accepted {
            execution_trace_id,
            sequence,
            trader_id,
            symbol,
            side,
            quantity,
            price,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_fill(
        &mut self,
        execution_trace_id: u64,
        sequence: u64,
        maker_trader_id: u64,
        taker_trader_id: u64,
        symbol: String,
        quantity: u64,
        price: i64,
        liquidity_side: LiquiditySide,
    ) {
        self.events.push(TimelineEvent::Fill {
            execution_trace_id,
            sequence,
            maker_trader_id,
            taker_trader_id,
            symbol,
            quantity,
            price,
            liquidity_side,
        });
    }

    pub fn record_update(
        &mut self,
        execution_trace_id: u64,
        sequence: u64,
        symbol: String,
        queue_depth: usize,
    ) {
        self.events.push(TimelineEvent::Update {
            execution_trace_id,
            sequence,
            symbol,
            queue_depth,
        });
    }

    pub fn record_reject(
        &mut self,
        execution_trace_id: u64,
        sequence: u64,
        trader_id: u64,
        reason: String,
    ) {
        self.events.push(TimelineEvent::Reject {
            execution_trace_id,
            sequence,
            trader_id,
            reason,
        });
    }

    pub fn record_cancel(
        &mut self,
        execution_trace_id: u64,
        sequence: u64,
        trader_id: u64,
        symbol: String,
    ) {
        self.events.push(TimelineEvent::Cancel {
            execution_trace_id,
            sequence,
            trader_id,
            symbol,
        });
    }

    pub fn export_ascii_timeline(&self, out: &mut impl std::io::Write) -> std::io::Result<()> {
        for ev in &self.events {
            match ev {
                TimelineEvent::Accepted {
                    execution_trace_id,
                    sequence,
                    trader_id,
                    symbol,
                    side,
                    quantity,
                    price,
                } => {
                    writeln!(
                        out,
                        "TRACE {:<5} | SEQ {} | ACCEPT | trader={} | {} | {} {} @ {}",
                        execution_trace_id, sequence, trader_id, symbol, side, quantity, price
                    )?;
                }
                TimelineEvent::Fill {
                    execution_trace_id,
                    sequence,
                    maker_trader_id,
                    taker_trader_id,
                    symbol: _,
                    quantity,
                    price,
                    liquidity_side,
                } => {
                    if *liquidity_side == LiquiditySide::Taker {
                        // Avoid printing twice if we want a clean timeline
                        writeln!(
                            out,
                            "TRACE {:<5} | SEQ {} | FILL   | maker={} taker={} qty={} px={}",
                            execution_trace_id,
                            sequence,
                            maker_trader_id,
                            taker_trader_id,
                            quantity,
                            price
                        )?;
                    }
                }
                TimelineEvent::Update {
                    execution_trace_id,
                    sequence,
                    symbol: _,
                    queue_depth,
                } => {
                    writeln!(
                        out,
                        "TRACE {:<5} | SEQ {} | UPDATE | queue_depth={}",
                        execution_trace_id, sequence, queue_depth
                    )?;
                }
                TimelineEvent::Reject {
                    execution_trace_id,
                    sequence,
                    trader_id: _,
                    reason,
                } => {
                    writeln!(
                        out,
                        "TRACE {:<5} | SEQ {} | REJECT | {}",
                        execution_trace_id, sequence, reason
                    )?;
                }
                TimelineEvent::Cancel {
                    execution_trace_id,
                    sequence,
                    trader_id,
                    symbol,
                } => {
                    writeln!(
                        out,
                        "TRACE {:<5} | SEQ {} | CANCEL | trader={} | {}",
                        execution_trace_id, sequence, trader_id, symbol
                    )?;
                }
            }
        }
        Ok(())
    }
}
