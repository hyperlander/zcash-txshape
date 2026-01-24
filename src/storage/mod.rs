//! SQLite storage for aggregate shape statistics (no tx hashes or addresses).

use crate::model::ShapeStats;
use rusqlite::Connection;
use std::path::Path;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS block_shapes (
    height INTEGER NOT NULL,
    n_txs INTEGER NOT NULL,
    vin_vout_hist TEXT NOT NULL,
    size_bucket_hist TEXT NOT NULL,
    version_hist TEXT NOT NULL,
    with_transparent INTEGER NOT NULL,
    with_shielded INTEGER NOT NULL,
    size_entropy REAL NOT NULL,
    PRIMARY KEY (height)
);

CREATE TABLE IF NOT EXISTS range_stats (
    range_low INTEGER NOT NULL,
    range_high INTEGER NOT NULL,
    n_txs INTEGER NOT NULL,
    vin_vout_hist TEXT NOT NULL,
    size_bucket_hist TEXT NOT NULL,
    version_hist TEXT NOT NULL,
    with_transparent INTEGER NOT NULL,
    with_shielded INTEGER NOT NULL,
    size_entropy REAL NOT NULL,
    PRIMARY KEY (range_low, range_high)
);
";

pub fn open_db(path: &Path) -> anyhow::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
}

pub fn upsert_block_stats(
    conn: &Connection,
    height: u32,
    stats: &ShapeStats,
) -> anyhow::Result<()> {
    let vin_vout = serde_json::to_string(&stats.vin_vout_hist)?;
    let size_hist = serde_json::to_string(&stats.size_bucket_hist)?;
    let version_hist = serde_json::to_string(&stats.version_hist)?;
    conn.execute(
        "INSERT INTO block_shapes (height, n_txs, vin_vout_hist, size_bucket_hist, version_hist, with_transparent, with_shielded, size_entropy)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(height) DO UPDATE SET
         n_txs=excluded.n_txs, vin_vout_hist=excluded.vin_vout_hist, size_bucket_hist=excluded.size_bucket_hist,
         version_hist=excluded.version_hist, with_transparent=excluded.with_transparent, with_shielded=excluded.with_shielded, size_entropy=excluded.size_entropy",
        rusqlite::params![
            height as i64,
            stats.n_txs as i64,
            vin_vout,
            size_hist,
            version_hist,
            stats.with_transparent as i64,
            stats.with_shielded as i64,
            stats.size_entropy,
        ],
    )?;
    Ok(())
}

pub fn get_block_stats(conn: &Connection, height: u32) -> anyhow::Result<Option<ShapeStats>> {
    let mut stmt = conn.prepare(
        "SELECT n_txs, vin_vout_hist, size_bucket_hist, version_hist, with_transparent, with_shielded, size_entropy FROM block_shapes WHERE height = ?1",
    )?;
    let mut rows = stmt.query([height as i64])?;
    if let Some(row) = rows.next()? {
        let vin_vout: String = row.get(1)?;
        let size_bucket_hist: String = row.get(2)?;
        let version_hist: String = row.get(3)?;
        let stats = ShapeStats {
            n_txs: row.get::<_, i64>(0)? as u64,
            vin_vout_hist: serde_json::from_str(&vin_vout)?,
            size_bucket_hist: serde_json::from_str(&size_bucket_hist)?,
            version_hist: serde_json::from_str(&version_hist)?,
            with_transparent: row.get::<_, i64>(4)? as u64,
            with_shielded: row.get::<_, i64>(5)? as u64,
            size_entropy: row.get(6)?,
        };
        return Ok(Some(stats));
    }
    Ok(None)
}

pub fn get_range_stats(
    conn: &Connection,
    low: u32,
    high: u32,
) -> anyhow::Result<Option<ShapeStats>> {
    let mut stmt = conn.prepare(
        "SELECT n_txs, vin_vout_hist, size_bucket_hist, version_hist, with_transparent, with_shielded, size_entropy FROM range_stats WHERE range_low = ?1 AND range_high = ?2",
    )?;
    let mut rows = stmt.query(rusqlite::params![low as i64, high as i64])?;
    if let Some(row) = rows.next()? {
        let vin_vout: String = row.get(1)?;
        let size_bucket_hist: String = row.get(2)?;
        let version_hist: String = row.get(3)?;
        let stats = ShapeStats {
            n_txs: row.get::<_, i64>(0)? as u64,
            vin_vout_hist: serde_json::from_str(&vin_vout)?,
            size_bucket_hist: serde_json::from_str(&size_bucket_hist)?,
            version_hist: serde_json::from_str(&version_hist)?,
            with_transparent: row.get::<_, i64>(4)? as u64,
            with_shielded: row.get::<_, i64>(5)? as u64,
            size_entropy: row.get(6)?,
        };
        return Ok(Some(stats));
    }
    Ok(None)
}

pub fn save_range_stats(
    conn: &Connection,
    low: u32,
    high: u32,
    stats: &ShapeStats,
) -> anyhow::Result<()> {
    let vin_vout = serde_json::to_string(&stats.vin_vout_hist)?;
    let size_hist = serde_json::to_string(&stats.size_bucket_hist)?;
    let version_hist = serde_json::to_string(&stats.version_hist)?;
    conn.execute(
        "INSERT INTO range_stats (range_low, range_high, n_txs, vin_vout_hist, size_bucket_hist, version_hist, with_transparent, with_shielded, size_entropy)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(range_low, range_high) DO UPDATE SET
         n_txs=excluded.n_txs, vin_vout_hist=excluded.vin_vout_hist, size_bucket_hist=excluded.size_bucket_hist,
         version_hist=excluded.version_hist, with_transparent=excluded.with_transparent, with_shielded=excluded.with_shielded, size_entropy=excluded.size_entropy",
        rusqlite::params![
            low as i64,
            high as i64,
            stats.n_txs as i64,
            vin_vout,
            size_hist,
            version_hist,
            stats.with_transparent as i64,
            stats.with_shielded as i64,
            stats.size_entropy,
        ],
    )?;
    Ok(())
}

pub fn block_heights_in_range(conn: &Connection, low: u32, high: u32) -> anyhow::Result<Vec<u32>> {
    let mut stmt = conn.prepare(
        "SELECT height FROM block_shapes WHERE height >= ?1 AND height < ?2 ORDER BY height",
    )?;
    let rows = stmt.query_map(rusqlite::params![low as i64, high as i64], |r| {
        r.get::<_, i64>(0)
    })?;
    let mut out = Vec::new();
    for h in rows {
        out.push(h? as u32);
    }
    Ok(out)
}

/// Build aggregate ShapeStats from per-block stats in the DB for a range (no tx hashes used).
pub fn aggregate_block_stats_in_range(
    conn: &Connection,
    low: u32,
    high: u32,
) -> anyhow::Result<ShapeStats> {
    let mut stmt = conn.prepare(
        "SELECT n_txs, vin_vout_hist, size_bucket_hist, version_hist, with_transparent, with_shielded FROM block_shapes WHERE height >= ?1 AND height < ?2",
    )?;
    let rows = stmt.query_map(rusqlite::params![low as i64, high as i64], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
        ))
    })?;
    let mut n_txs = 0u64;
    let mut vin_vout_hist = std::collections::HashMap::new();
    let mut size_bucket_hist = [0u64; 6];
    let mut version_hist = std::collections::HashMap::new();
    let mut with_transparent = 0u64;
    let mut with_shielded = 0u64;

    for row in rows {
        let (nt, vv, sh, vh, wt, ws): (i64, String, String, String, i64, i64) = row?;
        n_txs += nt as u64;
        with_transparent += wt as u64;
        with_shielded += ws as u64;
        let vv_map: std::collections::HashMap<String, u64> =
            serde_json::from_str(&vv).unwrap_or_default();
        for (k, v) in vv_map {
            *vin_vout_hist.entry(k).or_insert(0) += v;
        }
        let sb: [u64; 6] = serde_json::from_str(&sh).unwrap_or([0; 6]);
        for (i, &v) in sb.iter().enumerate() {
            size_bucket_hist[i] += v;
        }
        let vh_map: std::collections::HashMap<u32, u64> =
            serde_json::from_str(&vh).unwrap_or_default();
        for (k, v) in vh_map {
            *version_hist.entry(k).or_insert(0) += v;
        }
    }
    let size_entropy = crate::util::entropy(&size_bucket_hist);
    Ok(ShapeStats {
        n_txs,
        vin_vout_hist,
        size_bucket_hist,
        version_hist,
        with_transparent,
        with_shielded,
        size_entropy,
    })
}
