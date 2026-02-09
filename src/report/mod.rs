//! Reporting: daily/weekly summaries and range diffs.

use crate::model::ShapeStats;
use crate::storage;
use rusqlite::Connection;
use serde::Serialize;

#[derive(Serialize)]
struct SummaryReport {
    title: String,
    height_start: u32,
    height_end: u32,
    n_txs: u64,
    with_transparent: u64,
    with_shielded: u64,
    size_entropy: f64,
    version_hist: std::collections::HashMap<u32, u64>,
}

#[derive(Serialize)]
struct DiffReport {
    range_a: RangeStats,
    range_b: RangeStats,
    n_txs_delta: i64,
    with_transparent_delta: i64,
    with_shielded_delta: i64,
    size_entropy_delta: f64,
}

#[derive(Serialize)]
struct RangeStats {
    low: u32,
    high: u32,
    n_txs: u64,
    with_transparent: u64,
    with_shielded: u64,
    size_entropy: f64,
}

pub fn daily_summary(conn: &Connection, days: u32, json: bool) -> anyhow::Result<()> {
    let heights = storage::block_heights_in_range(conn, 0, u32::MAX)?;
    let max_h = heights.last().copied().unwrap_or(0);
    if max_h == 0 {
        if json {
            println!(
                "{}",
                serde_json::json!({"error": "no block data in database"})
            );
        } else {
            println!("No block data in database.");
        }
        return Ok(());
    }
    let blocks_per_day = 24 * 6;
    let start = max_h.saturating_sub(days * blocks_per_day);
    let stats = storage::aggregate_block_stats_in_range(conn, start, max_h)?;
    let title = format!("Last {} days (heights {}-{})", days, start, max_h);
    if json {
        let report = SummaryReport {
            title: title.clone(),
            height_start: start,
            height_end: max_h,
            n_txs: stats.n_txs,
            with_transparent: stats.with_transparent,
            with_shielded: stats.with_shielded,
            size_entropy: stats.size_entropy,
            version_hist: stats.version_hist.clone(),
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_stats_summary(&title, &stats);
    }
    Ok(())
}

pub fn weekly_summary(conn: &Connection, json: bool) -> anyhow::Result<()> {
    let heights = storage::block_heights_in_range(conn, 0, u32::MAX)?;
    let max_h = heights.last().copied().unwrap_or(0);
    if max_h == 0 {
        if json {
            println!(
                "{}",
                serde_json::json!({"error": "no block data in database"})
            );
        } else {
            println!("No block data in database.");
        }
        return Ok(());
    }
    const BLOCKS_PER_WEEK: u32 = 7 * 24 * 6;
    let start = max_h.saturating_sub(BLOCKS_PER_WEEK);
    let stats = storage::aggregate_block_stats_in_range(conn, start, max_h)?;
    let title = format!("Last week (heights {}-{})", start, max_h);
    if json {
        let report = SummaryReport {
            title: title.clone(),
            height_start: start,
            height_end: max_h,
            n_txs: stats.n_txs,
            with_transparent: stats.with_transparent,
            with_shielded: stats.with_shielded,
            size_entropy: stats.size_entropy,
            version_hist: stats.version_hist.clone(),
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_stats_summary(&title, &stats);
    }
    Ok(())
}

pub fn range_diff(
    conn: &Connection,
    a_lo: u32,
    a_hi: u32,
    b_lo: u32,
    b_hi: u32,
    json: bool,
) -> anyhow::Result<()> {
    let blocks_a = storage::block_heights_in_range(conn, a_lo, a_hi)?;
    let blocks_b = storage::block_heights_in_range(conn, b_lo, b_hi)?;
    let has_data_a = !blocks_a.is_empty();
    let has_data_b = !blocks_b.is_empty();

    let stats_a = storage::aggregate_block_stats_in_range(conn, a_lo, a_hi)?;
    let stats_b = storage::aggregate_block_stats_in_range(conn, b_lo, b_hi)?;

    let n_txs_delta = stats_b.n_txs as i64 - stats_a.n_txs as i64;
    let with_transparent_delta = stats_b.with_transparent as i64 - stats_a.with_transparent as i64;
    let with_shielded_delta = stats_b.with_shielded as i64 - stats_a.with_shielded as i64;
    let size_entropy_delta = stats_b.size_entropy - stats_a.size_entropy;

    if json {
        let report = DiffReport {
            range_a: RangeStats {
                low: a_lo,
                high: a_hi,
                n_txs: stats_a.n_txs,
                with_transparent: stats_a.with_transparent,
                with_shielded: stats_a.with_shielded,
                size_entropy: stats_a.size_entropy,
            },
            range_b: RangeStats {
                low: b_lo,
                high: b_hi,
                n_txs: stats_b.n_txs,
                with_transparent: stats_b.with_transparent,
                with_shielded: stats_b.with_shielded,
                size_entropy: stats_b.size_entropy,
            },
            n_txs_delta,
            with_transparent_delta,
            with_shielded_delta,
            size_entropy_delta,
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        if has_data_a {
            println!(
                "Range A [{}, {}): {} txs, with_transparent={}, with_shielded={}, size_entropy={:.4}",
                a_lo, a_hi, stats_a.n_txs, stats_a.with_transparent, stats_a.with_shielded, stats_a.size_entropy
            );
        } else {
            println!(
                "Range A [{}, {}): no block data in database (run collect --range {}..{} first)",
                a_lo, a_hi, a_lo, a_hi
            );
        }
        if has_data_b {
            println!(
                "Range B [{}, {}): {} txs, with_transparent={}, with_shielded={}, size_entropy={:.4}",
                b_lo, b_hi, stats_b.n_txs, stats_b.with_transparent, stats_b.with_shielded, stats_b.size_entropy
            );
        } else {
            println!(
                "Range B [{}, {}): no block data in database (run collect --range {}..{} first)",
                b_lo, b_hi, b_lo, b_hi
            );
        }
        if has_data_a || has_data_b {
            println!(
                "Diff: n_txs delta={}, with_transparent delta={}, with_shielded delta={}, size_entropy delta={:.4}",
                n_txs_delta, with_transparent_delta, with_shielded_delta, size_entropy_delta
            );
        } else {
            println!("Diff: no data to compare (collect block data for both ranges first).");
        }
    }
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
