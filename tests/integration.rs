//! Integration tests: storage and report with in-memory DB.

use zcash_txshape::model::{ShapeStats, TxShape};
use zcash_txshape::report;
use zcash_txshape::storage;
use zcash_txshape::util::size_bucket;

fn sample_stats() -> ShapeStats {
    let shapes = vec![
        TxShape {
            n_vin: 1,
            n_vout: 2,
            n_joinsplit: 0,
            n_sapling_spend: 0,
            n_sapling_output: 0,
            n_orchard_action: 0,
            size_bucket: size_bucket(300),
            version: 4,
        },
        TxShape {
            n_vin: 0,
            n_vout: 0,
            n_joinsplit: 0,
            n_sapling_spend: 1,
            n_sapling_output: 1,
            n_orchard_action: 0,
            size_bucket: size_bucket(500),
            version: 4,
        },
    ];
    ShapeStats::from_shapes(&shapes)
}

#[test]
fn storage_upsert_and_get_block_stats() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let conn = storage::open_db(&db_path).unwrap();
    let stats = sample_stats();
    storage::upsert_block_stats(&conn, 100, &stats).unwrap();
    let loaded = storage::get_block_stats(&conn, 100).unwrap().unwrap();
    assert_eq!(loaded.n_txs, stats.n_txs);
    assert_eq!(loaded.with_transparent, 1);
    assert_eq!(loaded.with_shielded, 1);
}

#[test]
fn storage_aggregate_range() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let conn = storage::open_db(&db_path).unwrap();
    let stats1 = sample_stats();
    storage::upsert_block_stats(&conn, 10, &stats1).unwrap();
    let stats2 = sample_stats();
    storage::upsert_block_stats(&conn, 11, &stats2).unwrap();
    let agg = storage::aggregate_block_stats_in_range(&conn, 10, 12).unwrap();
    assert_eq!(agg.n_txs, 2 + 2);
}

#[test]
fn report_daily_empty_db() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let conn = storage::open_db(&db_path).unwrap();
    let result = report::daily_summary(&conn, 7);
    result.unwrap();
}

#[test]
fn report_range_diff() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let conn = storage::open_db(&db_path).unwrap();
    let stats = sample_stats();
    storage::upsert_block_stats(&conn, 0, &stats).unwrap();
    storage::upsert_block_stats(&conn, 1, &stats).unwrap();
    let result = report::range_diff(&conn, 0, 1, 1, 2);
    result.unwrap();
}
