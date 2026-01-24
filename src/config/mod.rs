//! Configuration load and validation.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

/// Application configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub node: NodeConfig,
    pub storage: StorageConfig,
    pub collector: CollectorConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeConfig {
    /// zcashd RPC URL (e.g. http://127.0.0.1:8232).
    pub rpc_url: String,
    /// Optional RPC username (for HTTP basic auth).
    pub rpc_user: Option<String>,
    /// Optional RPC password.
    pub rpc_password: Option<String>,
    /// Request timeout in seconds.
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_timeout_secs() -> u64 {
    30
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    /// Path to SQLite database file.
    pub db_path: std::path::PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CollectorConfig {
    /// Maximum number of blocks to request per batch (rate limiting).
    #[serde(default = "default_batch_size")]
    pub batch_size: u32,
    /// Delay in milliseconds between batch requests.
    #[serde(default = "default_batch_delay_ms")]
    pub batch_delay_ms: u64,
}

fn default_batch_size() -> u32 {
    10
}

fn default_batch_delay_ms() -> u64 {
    500
}

impl Config {
    /// Load and validate config from a TOML file.
    pub fn load(path: &Path) -> Result<Config> {
        let data = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config: {}", path.display()))?;
        let config: Config = toml::from_str(&data).context("invalid config TOML")?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.node.rpc_url.is_empty() {
            anyhow::bail!("node.rpc_url must be non-empty");
        }
        if self.collector.batch_size == 0 {
            anyhow::bail!("collector.batch_size must be positive");
        }
        Ok(())
    }
}

/// Default config for use when no file is present (e.g. documentation).
pub fn default_config_toml() -> &'static str {
    r#"
# Zcash node RPC (read-only). Use a local zcashd or a trusted endpoint.
[node]
rpc_url = "http://127.0.0.1:8232"
# rpc_user = "user"
# rpc_password = "pass"
timeout_secs = 30

[storage]
db_path = "txshape.db"

[collector]
batch_size = 10
batch_delay_ms = 500
"#
}
