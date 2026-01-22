# zcash-txshape

**Transaction Shape Analyzer for Zcash (Read-Only, Aggregate, Non-Attributing)**

A production-quality Rust tool that analyzes *transaction shape consistency* on the Zcash blockchain. It does not perform privacy analysis, deanonymization, or infrastructure monitoring.

## Motivation

Transaction *shape*—the counts of inputs and outputs, presence of transparent vs shielded components, size buckets, and format flags—can become more uniform or more diverse over time. If new wallet or SDK releases unintentionally produce a narrow set of shapes, or protocol upgrades change statistical composition, transactions may become distinguishable at the metadata level. This tool helps researchers and developers understand those aggregate trends without touching addresses, values, or identities.

## Non-Goals and Threat Model

- **Not privacy analysis.** We do not estimate anonymity sets or link transactions to users.
- **Not deanonymization.** No addresses or transaction hashes are persisted; no user linkage.
- **Read-only and passive.** The tool only consumes public blockchain data and writes aggregate statistics.
- **Threat model:** We assume the operator has read-only access to a Zcash node or lightwalletd. We do not store data that could attribute activity to individuals. See [SECURITY.md](SECURITY.md) for details.

## How to Run Locally

Requires a Zcash node (zcashd) with RPC enabled, or a compatible endpoint. See configuration below.

```bash
cargo build --release
./target/release/zcash-txshape --config config.toml collect --range 0..1000
./target/release/zcash-txshape report daily --days 7
```

## How to Extend

- **Config:** Add options in `internal/config` and validate on startup.
- **Collector:** Implement additional backends (e.g. lightwalletd gRPC) behind the same shape-extraction interface.
- **Model:** Extend histograms and entropy metrics in `internal/model`.
- **Reporting:** Add output formats in `internal/report`.

See [CONTRIBUTING.md](CONTRIBUTING.md) for development workflow.

## License

MIT. See [LICENSE](LICENSE).
