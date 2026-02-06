# zcash-txshape

**Transaction Shape Analyzer for Zcash (Read-Only, Aggregate, Non-Attributing)**

A production-quality Rust tool that analyzes *transaction shape consistency* on the Zcash blockchain. It does not perform privacy analysis, deanonymization, or infrastructure monitoring.

## Motivation

Transaction *shape*—the counts of inputs and outputs, presence of transparent vs shielded components, size buckets, and format flags—can become more uniform or more diverse over time. If new wallet or SDK releases unintentionally produce a narrow set of shapes, or protocol upgrades change statistical composition, transactions may become distinguishable at the metadata level. This tool helps researchers and developers understand those aggregate trends without touching addresses, values, or identities.

## How This Differs From Privacy Analysis

Privacy analysis typically studies *who* might be linked to a transaction (anonymity sets, graph analysis, timing). This tool does **not** do that. It studies *whether transaction metadata is becoming more or less uniform over time*—a statistical, aggregate question. The goal is to detect accidental fingerprinting (e.g. one wallet emitting a very distinctive shape distribution), not to attribute activity to users.

## Non-Goals and Threat Model

- **Not privacy analysis.** We do not estimate anonymity sets or link transactions to users.
- **Not deanonymization.** No addresses or transaction hashes are persisted; no user linkage.
- **Read-only and passive.** The tool only consumes public blockchain data and writes aggregate statistics.
- **Threat model:** We assume the operator has read-only access to a Zcash node or lightwalletd. We do not store data that could attribute activity to individuals. See [SECURITY.md](SECURITY.md) for details.

## How to Run Locally

Requires a Zcash node (zcashd) with RPC enabled, or a compatible endpoint.

```bash
cargo build --release
./target/release/zcash-txshape --config config.toml collect --range 0..1000
./target/release/zcash-txshape report daily --days 7
./target/release/zcash-txshape report daily --days 7 --output json   # JSON for researchers
```

## Configuration

Copy `config.toml` and set `node.rpc_url` to your zcashd RPC endpoint (e.g. `http://127.0.0.1:8232`). Optionally set `rpc_user` and `rpc_password` if your node requires auth. Config path can be overridden with `--config` or the `ZCASH_TXSHAPE_CONFIG` environment variable. The file is validated on startup.

For mainnet, run a local zcashd (e.g. `zcashd -daemon`) and ensure RPC is bound (e.g. `rpcallowip=127.0.0.1` in zcash.conf). No public default RPC endpoint is shipped; use your own node or a trusted service.

## Docker

```bash
docker build -t zcash-txshape .
docker run --rm -v /path/to/data:/data -e ZCASH_TXSHAPE_CONFIG=/data/config.toml zcash-txshape report daily --days 7
```

Mount a directory containing `config.toml` and (optionally) the SQLite DB path used in that config.

## How to Extend

- **Config:** Add options in `src/config` and validate on startup.
- **Collector:** Implement additional backends (e.g. lightwalletd gRPC) behind the same shape-extraction interface.
- **Model:** Extend histograms and entropy metrics in `src/model`.
- **Reporting:** Add output formats (e.g. CSV) in `src/report`; use `--output json` for machine-readable reports.

See [CONTRIBUTING.md](CONTRIBUTING.md) for development workflow.

## License

MIT. See [LICENSE](LICENSE).
