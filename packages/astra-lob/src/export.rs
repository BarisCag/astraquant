use crate::diagnostics::ReplayDiagnostics;
use crate::snapshot::BookSnapshot;
use crate::types::TradeExecution;
use std::io::{self, Write};

pub fn export_trades_csv<W: Write>(writer: &mut W, trades: &[TradeExecution]) -> io::Result<()> {
    writeln!(
        writer,
        "symbol,resting_order_id,aggressive_order_id,match_price,matched_quantity,liquidity_side,timestamp_ns"
    )?;
    for t in trades {
        let side = match t.liquidity_side {
            crate::types::LiquiditySide::Maker => "Maker",
            crate::types::LiquiditySide::Taker => "Taker",
        };
        writeln!(
            writer,
            "{},{},{},{},{},{},{}",
            t.symbol,
            t.resting_order_id,
            t.aggressive_order_id,
            t.match_price.0,
            t.matched_quantity.0,
            side,
            t.timestamp_ns
        )?;
    }
    Ok(())
}

pub fn export_book_snapshot_csv<W: Write>(
    writer: &mut W,
    snapshot: &BookSnapshot,
) -> io::Result<()> {
    writeln!(
        writer,
        "symbol,best_bid,best_ask,spread_ticks,total_bid_levels,total_ask_levels,total_bid_liquidity,total_ask_liquidity"
    )?;
    let bid = snapshot
        .best_bid
        .map(|p| p.0.to_string())
        .unwrap_or_default();
    let ask = snapshot
        .best_ask
        .map(|p| p.0.to_string())
        .unwrap_or_default();
    writeln!(
        writer,
        "{},{},{},{},{},{},{},{}",
        snapshot.symbol,
        bid,
        ask,
        snapshot.spread_ticks,
        snapshot.total_bid_levels,
        snapshot.total_ask_levels,
        snapshot.total_bid_liquidity.0,
        snapshot.total_ask_liquidity.0
    )?;
    Ok(())
}

pub fn export_replay_diagnostics_csv<W: Write>(
    writer: &mut W,
    diag: &ReplayDiagnostics,
) -> io::Result<()> {
    writeln!(writer, "total_orders,total_trades,total_cancels,total_rejections,total_partial_fills,total_full_fills,total_integrity_violations,peak_bid_depth,peak_ask_depth,max_queue_length")?;
    writeln!(
        writer,
        "{},{},{},{},{},{},{},{},{},{}",
        diag.total_orders,
        diag.total_trades,
        diag.total_cancels,
        diag.total_rejections,
        diag.total_partial_fills,
        diag.total_full_fills,
        diag.total_integrity_violations,
        diag.peak_bid_depth,
        diag.peak_ask_depth,
        diag.max_queue_length
    )?;
    Ok(())
}
