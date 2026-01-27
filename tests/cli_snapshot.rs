//! CLI snapshot tests: run binary and check output.

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_zcash-txshape"))
}

#[test]
fn cli_help() {
    let out = bin().arg("--help").output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("zcash-txshape"));
    assert!(stdout.contains("Transaction Shape Analyzer"));
    assert!(stdout.contains("collect"));
    assert!(stdout.contains("report"));
}

#[test]
fn cli_report_help() {
    let out = bin().args(["report", "--help"]).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("daily"));
    assert!(stdout.contains("weekly"));
    assert!(stdout.contains("diff"));
}

#[test]
fn cli_report_daily_empty_db() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("snap.db");
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        format!(
            r#"
[node]
rpc_url = "http://127.0.0.1:8232"
timeout_secs = 30

[storage]
db_path = "{}"

[collector]
batch_size = 5
batch_delay_ms = 100
"#,
            db_path.display()
        ),
    )
    .unwrap();
    let out = bin()
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "report",
            "daily",
            "--days",
            "7",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("No block data") || stdout.contains("n_txs"));
}
