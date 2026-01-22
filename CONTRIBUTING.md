# Contributing to zcash-txshape

Thank you for your interest in improving the Transaction Shape Analyzer for Zcash.

## Development Setup

- Rust stable toolchain.
- Run `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo build --release` before submitting changes.
- No placeholders: no TODO/FIXME/stub/dummy; every change must compile and do meaningful work.

## Scope and Norms

- The tool is **read-only and passive**. It does not analyze privacy, anonymity sets, or user behavior; it analyzes **transaction shape consistency** (aggregate metadata).
- Do not add features that store addresses, transaction hashes long-term, or any user-attributing data.
- Prefer conservative, well-documented choices; when in doubt, align with Zcash community norms and the project’s threat model (see [SECURITY.md](SECURITY.md)).

## Workflow

1. Open an issue or discuss a change idea.
2. Fork, branch, implement, and add or update tests as needed.
3. Ensure all checks pass (fmt, clippy, test, release build).
4. Submit a pull request with a clear description. Use Conventional Commits (`feat:`, `fix:`, `docs:`, etc.) in commit messages.

## Code Style

- Follow `cargo fmt` and `cargo clippy` output.
- Prefer no `unsafe` unless strictly necessary and justified in comments.
- Use the existing module layout: config, collector, model, storage, report, util.
