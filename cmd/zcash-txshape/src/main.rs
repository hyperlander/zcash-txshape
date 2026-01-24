//! Binary entrypoint for zcash-txshape.

use clap::Parser;
use std::path::PathBuf;
use zcash_txshape::collector;
use zcash_txshape::config::Config;
use zcash_txshape::report;
use zcash_txshape::storage;

#[derive(clap::Parser)]
#[command(name = "zcash-txshape", about = "Transaction Shape Analyzer for Zcash")]
struct Cli {
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Collect block data and compute shape statistics.
    Collect {
        /// Block range (e.g. 0..1000 or 50000..50100).
        #[arg(long)]
        range: String,
    },
    /// Produce reports from stored statistics.
    Report {
        #[command(subcommand)]
        kind: ReportKind,
    },
}

#[derive(clap::Subcommand)]
enum ReportKind {
    /// Daily summary for the last N days.
    Daily {
        #[arg(long, default_value = "7")]
        days: u32,
    },
    /// Weekly summary.
    Weekly,
    /// Diff between two block ranges (e.g. 0..1000 vs 1000..2000).
    Diff {
        #[arg(long)]
        range_a: String,
        #[arg(long)]
        range_b: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config_path = cli
        .config
        .or_else(|| {
            std::env::var("ZCASH_TXSHAPE_CONFIG")
                .ok()
                .map(PathBuf::from)
        })
        .unwrap_or_else(|| PathBuf::from("config.toml"));
    let config = Config::load(&config_path)?;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("zcash_txshape=info".parse()?),
        )
        .init();

    match cli.command {
        Command::Collect { range } => {
            let (low, high) = parse_range(&range)?;
            let db = storage::open_db(&config.storage.db_path)?;
            collector::run_collect(&config, &db, low, high).await?;
        }
        Command::Report { kind } => {
            let db = storage::open_db(&config.storage.db_path)?;
            match kind {
                ReportKind::Daily { days } => report::daily_summary(&db, days)?,
                ReportKind::Weekly => report::weekly_summary(&db)?,
                ReportKind::Diff { range_a, range_b } => {
                    let (a_lo, a_hi) = parse_range(&range_a)?;
                    let (b_lo, b_hi) = parse_range(&range_b)?;
                    report::range_diff(&db, a_lo, a_hi, b_lo, b_hi)?;
                }
            }
        }
    }
    Ok(())
}

fn parse_range(s: &str) -> anyhow::Result<(u32, u32)> {
    let s = s.trim();
    let (a, b) = s
        .split_once("..")
        .ok_or_else(|| anyhow::anyhow!("range must be of form START..END"))?;
    let low: u32 = a
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid range start"))?;
    let high: u32 = b
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid range end"))?;
    if low >= high {
        anyhow::bail!("range start must be less than end");
    }
    Ok((low, high))
}
