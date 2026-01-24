//! Reporting: daily/weekly summaries and range diffs.

use crate::model::ShapeStats;
use crate::storage;
use rusqlite::Connection;

pub fn daily_summary(conn: &Connection, days: u32) -> anyhow::Result<()> {
    let heights = storage::block_heights_in_range(conn, 0, u32::MAX)?;
    let max_h = heights.last().copied().unwrap_or(0);
    if max_h == 0 {
        println!("No block data in database.");
        return Ok(());
    }
    let blocks_per_day = 24 * 6;
    let start = max_h.saturating_sub(days * blocks_per_day);
    let stats = storage::aggregate_block_stats_in_range(conn, start, max_h)?;
    print_stats_summary(
        &format!("Last {} days (heights {}-{})", days, start, max_h),
        &stats,
    );
    Ok(())
}

pub fn weekly_summary(conn: &Connection) -> anyhow::Result<()> {
    let heights = storage::block_heights_in_range(conn, 0, u32::MAX)?;
    let max_h = heights.last().copied().unwrap_or(0);
    if max_h == 0 {
        println!("No block data in database.");
        return Ok(());
    }
    const BLOCKS_PER_WEEK: u32 = 7 * 24 * 6;
    let start = max_h.saturating_sub(BLOCKS_PER_WEEK);
    let stats = storage::aggregate_block_stats_in_range(conn, start, max_h)?;
    print_stats_summary(&format!("Last week (heights {}-{})", start, max_h), &stats);
    Ok(())
}

pub fn range_diff(
    conn: &Connection,
    a_lo: u32,
    a_hi: u32,
    b_lo: u32,
    b_hi: u32,
) -> anyhow::Result<()> {
    let stats_a = storage::aggregate_block_stats_in_range(conn, a_lo, a_hi)?;
    let stats_b = storage::aggregate_block_stats_in_range(conn, b_lo, b_hi)?;
    println!(
        "Range A [{}, {}): {} txs, size_entropy={:.4}",
        a_lo, a_hi, stats_a.n_txs, stats_a.size_entropy
    );
    println!(
        "Range B [{}, {}): {} txs, size_entropy={:.4}",
        b_lo, b_hi, stats_b.n_txs, stats_b.size_entropy
    );
    println!(
        "Diff: n_txs delta={}, size_entropy delta={:.4}",
        stats_b.n_txs as i64 - stats_a.n_txs as i64,
        stats_b.size_entropy - stats_a.size_entropy
    );
    Ok(())
}

fn print_stats_summary(title: &str, stats: &ShapeStats) {
    println!("--- {} ---", title);
    println!("n_txs: {}", stats.n_txs);
    println!("with_transparent: {}", stats.with_transparent);
    println!("with_shielded: {}", stats.with_shielded);
    println!("size_entropy: {:.4}", stats.size_entropy);
    println!("version_hist: {:?}", stats.version_hist);
}
