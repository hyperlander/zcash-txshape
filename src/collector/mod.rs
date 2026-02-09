//! Block/transaction data collection (read-only). Extracts shape metadata only.

use crate::config::Config;
use crate::model::{ShapeStats, TxShape};
use crate::storage;
use crate::util::size_bucket;
use base64::Engine;
use rusqlite::Connection;
use serde::Deserialize;
use std::time::Duration;
use tracing::info;

/// zcashd getblock verbosity=2 response (subset we need).
#[derive(Debug, Deserialize)]
struct BlockResponse {
    #[allow(dead_code)]
    height: Option<u32>,
    tx: Option<Vec<TxResponse>>,
}

#[derive(Debug, Deserialize)]
struct TxResponse {
    size: Option<u32>,
    version: Option<u32>,
    vin: Option<Vec<serde_json::Value>>,
    vout: Option<Vec<serde_json::Value>>,
    vjoinsplit: Option<Vec<serde_json::Value>>,
    #[serde(rename = "vShieldedSpend")]
    v_shielded_spend: Option<Vec<serde_json::Value>>,
    #[serde(rename = "vShieldedOutput")]
    v_shielded_output: Option<Vec<serde_json::Value>>,
    orchard: Option<OrchardPart>,
}

#[derive(Debug, Deserialize)]
struct OrchardPart {
    actions: Option<Vec<serde_json::Value>>,
}

fn extract_shape(tx: &TxResponse) -> TxShape {
    let n_vin = tx.vin.as_ref().map(|v| v.len()).unwrap_or(0) as u32;
    let n_vout = tx.vout.as_ref().map(|v| v.len()).unwrap_or(0) as u32;
    let n_joinsplit = tx.vjoinsplit.as_ref().map(|v| v.len()).unwrap_or(0) as u32;
    let n_sapling_spend = tx.v_shielded_spend.as_ref().map(|v| v.len()).unwrap_or(0) as u32;
    let n_sapling_output = tx.v_shielded_output.as_ref().map(|v| v.len()).unwrap_or(0) as u32;
    let n_orchard_action = tx
        .orchard
        .as_ref()
        .and_then(|o| o.actions.as_ref())
        .map(|v| v.len())
        .unwrap_or(0) as u32;
    let size = tx.size.unwrap_or(0);
    let version = tx.version.unwrap_or(1);
    TxShape {
        n_vin,
        n_vout,
        n_joinsplit,
        n_sapling_spend,
        n_sapling_output,
        n_orchard_action,
        size_bucket: size_bucket(size),
        version,
    }
}

/// Run collection for block range [low, high). Reads from zcashd RPC, writes only aggregate stats.
pub async fn run_collect(
    config: &Config,
    db: &Connection,
    low: u32,
    high: u32,
) -> anyhow::Result<()> {
    let client = build_http_client(config)?;
    let batch_size = config.collector.batch_size;
    let delay = Duration::from_millis(config.collector.batch_delay_ms);

    let mut all_shapes: Vec<TxShape> = Vec::new();
    let mut block_count = 0u32;

    for start in (low..high).step_by(batch_size as usize) {
        let end = (start + batch_size).min(high);
        for height in start..end {
            match fetch_block_at_height(&client, config, height).await {
                Ok(Some(shapes)) => {
                    for s in &shapes {
                        all_shapes.push(s.clone());
                    }
                    if storage::get_block_stats(db, height)?.is_none() {
                        let stats = ShapeStats::from_shapes(&shapes);
                        storage::upsert_block_stats(db, height, &stats)?;
                    }
                    block_count += 1;
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(height, "fetch block failed: {}", e);
                }
            }
        }
        if end < high {
            tokio::time::sleep(delay).await;
        }
    }

    let range_stats = ShapeStats::from_shapes(&all_shapes);
    storage::save_range_stats(db, low, high, &range_stats)?;
    info!(
        low,
        high,
        blocks = block_count,
        n_txs = range_stats.n_txs,
        "collect done"
    );
    Ok(())
}

fn build_http_client(config: &Config) -> anyhow::Result<reqwest::Client> {
    let mut builder =
        reqwest::Client::builder().timeout(Duration::from_secs(config.node.timeout_secs));
    if config.node.rpc_user.is_some() || config.node.rpc_password.is_some() {
        let u = config.node.rpc_user.as_deref().unwrap_or("");
        let p = config.node.rpc_password.as_deref().unwrap_or("");
        let auth = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", u, p));
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Basic {}", auth))?,
        );
        builder = builder.default_headers(headers);
    }
    Ok(builder.build()?)
}

/// Try getblock by height first, then by hash (getblockhash + getblock) when height fails (e.g. 500 or null).
async fn fetch_block_at_height(
    client: &reqwest::Client,
    config: &Config,
    height: u32,
) -> anyhow::Result<Option<Vec<TxShape>>> {
    let by_height = fetch_block_params(client, config, serde_json::json!([height, 2])).await;
    match by_height {
        Ok(Some(s)) => return Ok(Some(s)),
        Ok(None) => {}
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("500") || msg.contains("501") || msg.contains("502") {
                tracing::debug!(
                    height,
                    "getblock by height failed ({}), trying getblockhash fallback",
                    msg
                );
            } else {
                return Err(e);
            }
        }
    }
    let hash_body = serde_json::json!({
        "jsonrpc": "1.0",
        "id": "txshape",
        "method": "getblockhash",
        "params": [height]
    });
    let hash_resp = client
        .post(&config.node.rpc_url)
        .json(&hash_body)
        .send()
        .await?;
    let hash_status = hash_resp.status();
    let hash_bytes = hash_resp.bytes().await?;
    if !hash_status.is_success() {
        return Ok(None);
    }
    let hash_json: serde_json::Value = serde_json::from_slice(&hash_bytes).ok().unwrap_or_default();
    let block_hash = match hash_json.get("result").and_then(|r| r.as_str()) {
        Some(h) => h,
        None => return Ok(None),
    };
    fetch_block_params(client, config, serde_json::json!([block_hash, 2])).await
}

/// Call getblock with given params (e.g. [height, 2] or ["hash", 2]).
async fn fetch_block_params(
    client: &reqwest::Client,
    config: &Config,
    params: serde_json::Value,
) -> anyhow::Result<Option<Vec<TxShape>>> {
    let body = serde_json::json!({
        "jsonrpc": "1.0",
        "id": "txshape",
        "method": "getblock",
        "params": params
    });
    let resp = client.post(&config.node.rpc_url).json(&body).send().await?;
    let status = resp.status();
    let body_bytes = resp.bytes().await?;
    if !status.is_success() {
        let msg = String::from_utf8_lossy(&body_bytes);
        anyhow::bail!("RPC status {}: {}", status, msg.trim());
    }
    let json: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let result_val = match json.get("result") {
        Some(r) => r.clone(),
        None => {
            if json.get("error").is_some() {
                return Ok(None);
            }
            anyhow::bail!("no result in RPC response");
        }
    };
    if result_val.is_null() {
        return Ok(None);
    }
    let block: BlockResponse = serde_json::from_value(result_val)?;
    let txs = match block.tx {
        Some(t) => t,
        None => return Ok(Some(Vec::new())),
    };
    let shapes: Vec<TxShape> = txs.iter().map(extract_shape).collect();
    Ok(Some(shapes))
}
