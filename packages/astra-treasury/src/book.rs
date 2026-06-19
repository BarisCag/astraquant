use std::collections::HashMap;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use astra_core::events::{AstraEvent, EventType, PayloadMetadata, PayloadEncoding};
use crate::types::{CashPosition, Currency};
use crate::event::TreasuryEvent;
use astra_paper::events::PaperEvent;
use astra_paper::types::Side;

pub struct TreasuryBook {
    pub positions: HashMap<Currency, CashPosition>,
    pub rfr_map: HashMap<Currency, Decimal>,
    pub liquidity_threshold_days: u32,
    pub daily_burn_rate: Decimal, // Used for runway calc if no pending outflows
}

impl TreasuryBook {
    pub fn new(liquidity_threshold_days: u32) -> Self {
        let mut rfr_map = HashMap::new();
        rfr_map.insert(Currency::USD, Decimal::from_f64(0.05).unwrap()); // 5% APR
        rfr_map.insert(Currency::EUR, Decimal::from_f64(0.03).unwrap()); // 3% APR
        rfr_map.insert(Currency::TRY, Decimal::from_f64(0.40).unwrap()); // 40% APR

        let mut positions = HashMap::new();
        positions.insert(Currency::USD, CashPosition {
            currency: Currency::USD,
            available_balance: Decimal::from(100_000),
            pending_inflows: Decimal::ZERO,
            pending_outflows: Decimal::ZERO,
            accrued_interest: Decimal::ZERO,
            next_settlement_days: 0,
            liquidity_runway_days: 30,
        });

        Self {
            positions,
            rfr_map,
            liquidity_threshold_days,
            daily_burn_rate: Decimal::from(500), // Base $500/day operational burn
        }
    }

    pub fn process_event(&mut self, in_event: &AstraEvent) -> Vec<AstraEvent> {
        let mut out_events = Vec::new();

        match in_event.event_type {
            EventType::OrderFilled => {
                if let Ok(PaperEvent::Fill(fill)) = astra_core::serialization::deserialize_canonical::<PaperEvent>(&in_event.payload) {
                    let currency = Currency::USD; // Hardcoding USD for BTCUSDT for simplicity
                    let notional = (fill.fill_quantity as u128 * fill.fill_price as u128) / 100_000_000;
                    let notional_dec = Decimal::from_u128(notional).unwrap() / Decimal::from(100_000_000);

                    let pos = self.positions.get_mut(&currency).unwrap();
                    match fill.side {
                        Side::Buy => {
                            pos.pending_outflows += notional_dec;
                        }
                        Side::Sell => {
                            pos.pending_inflows += notional_dec;
                        }
                    }
                    
                    // Reset or extend settlement to T+2
                    if pos.next_settlement_days < 2 {
                        pos.next_settlement_days = 2;
                    }

                    // Re-evaluate runway
                    let net_balance = pos.net_projected_balance();
                    let burn = if pos.pending_outflows > Decimal::ZERO {
                        pos.pending_outflows / Decimal::from(pos.next_settlement_days.max(1))
                    } else {
                        self.daily_burn_rate
                    };

                    if net_balance > Decimal::ZERO {
                        pos.liquidity_runway_days = (net_balance / burn).to_u32().unwrap_or(u32::MAX);
                    } else {
                        pos.liquidity_runway_days = 0;
                    }

                    let tevent = TreasuryEvent::CashPositionUpdate { position: pos.clone() };
                    out_events.push(Self::wrap_event(in_event.timestamp_ns, tevent));
                }
            }
            EventType::AuditCheckpoint => {
                // We use AuditCheckpoint as our EOD Tick signal
                let mut updates = Vec::new();

                for (curr, pos) in self.positions.iter_mut() {
                    let rfr = self.rfr_map.get(curr).copied().unwrap_or(Decimal::ZERO);
                    
                    // 1. Accrue Interest
                    let daily_rate = rfr / Decimal::from(365);
                    let accrued_today = pos.available_balance * daily_rate;
                    pos.accrued_interest += accrued_today;
                    
                    updates.push(Self::wrap_event(
                        in_event.timestamp_ns, 
                        TreasuryEvent::AccrualUpdate {
                            currency: *curr,
                            accrued_amount: accrued_today,
                        }
                    ));

                    // 2. Settlement Migration
                    if pos.next_settlement_days > 0 {
                        pos.next_settlement_days -= 1;
                        if pos.next_settlement_days == 0 {
                            pos.available_balance += pos.pending_inflows - pos.pending_outflows;
                            pos.pending_inflows = Decimal::ZERO;
                            pos.pending_outflows = Decimal::ZERO;
                        }
                    }

                    // 3. Re-evaluate runway
                    let net_balance = pos.net_projected_balance();
                    
                    let burn = if pos.pending_outflows > Decimal::ZERO {
                        pos.pending_outflows / Decimal::from(pos.next_settlement_days.max(1))
                    } else {
                        self.daily_burn_rate
                    };

                    if net_balance > Decimal::ZERO {
                        pos.liquidity_runway_days = (net_balance / burn).to_u32().unwrap_or(u32::MAX);
                    } else {
                        pos.liquidity_runway_days = 0;
                    }

                    updates.push(Self::wrap_event(
                        in_event.timestamp_ns,
                        TreasuryEvent::CashPositionUpdate { position: pos.clone() },
                    ));

                    if pos.liquidity_runway_days < self.liquidity_threshold_days {
                        updates.push(Self::wrap_event(
                            in_event.timestamp_ns,
                            TreasuryEvent::LiquidityWarning {
                                currency: *curr,
                                shortfall_amount: (Decimal::from(self.liquidity_threshold_days) * burn) - net_balance,
                                metric_breached: format!("LCR_{}_DAY_STRESS", self.liquidity_threshold_days),
                            }
                        ));
                    }
                }

                out_events.extend(updates);
            }
            _ => {}
        }

        out_events
    }

    fn wrap_event(timestamp_ns: u64, tevent: TreasuryEvent) -> AstraEvent {
        let payload = astra_core::serialization::serialize_canonical(&tevent).unwrap();
        // We register Treasury state updates under StateSnapshot.
        // The event payload uniquely identifies it as a TreasuryEvent.
        AstraEvent::new(
            timestamp_ns,
            0,
            EventType::StateSnapshot,
            payload,
            PayloadMetadata::new(PayloadEncoding::Bincode, 1),
        )
    }
}
