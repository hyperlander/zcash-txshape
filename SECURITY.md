# Security and Threat Model

## Scope

zcash-txshape is a **read-only, aggregate, non-attributing** transaction shape analyzer. It is designed to reduce privacy risk by construction.

## Threat Model

- **Assumed:** The operator has read-only access to a Zcash node (e.g. zcashd RPC) or a lightwalletd-compatible service. No write or wallet access is required.
- **Assumed:** Output (reports, database) is not used to link transactions to real-world identities. The tool does not store addresses or transaction hashes beyond in-memory processing.
- **Out of scope:** Protection against a malicious RPC endpoint; protection of the machine running the tool; secure distribution of binaries.

## Non-Goals

- This tool does **not** perform privacy or anonymity-set analysis.
- It does **not** deanonymize users or link transactions to addresses or identities.
- It does **not** store precise values; only bucketed size and fee ranges where applicable.
- It does **not** require or use wallet functionality.

## Data Retained

- **Stored:** Block height ranges, per-block and per-window aggregate shape statistics (e.g. histograms of input/output counts, size buckets, entropy estimates). No transaction IDs, no addresses, no precise amounts.
- **Not stored:** Transaction hashes (except transiently during a single block processing), addresses, key material, or user-identifying data.

## Reporting Vulnerabilities

If you believe you have found a security issue that could lead to attribution or misuse of data produced by this tool, please report it responsibly. Prefer private disclosure; we will acknowledge and work on fixes in a timely manner.
