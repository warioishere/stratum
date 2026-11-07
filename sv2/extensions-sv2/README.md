# extensions_sv2

[![crates.io](https://img.shields.io/crates/v/extensions_sv2.svg)](https://crates.io/crates/extensions_sv2)
[![docs.rs](https://docs.rs/extensions_sv2/badge.svg)](https://docs.rs/extensions_sv2)
[![rustc+](https://img.shields.io/badge/rustc-1.75.0%2B-lightgrey.svg)](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)
[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/stratum-mining/stratum/blob/main/LICENSE.md)
[![codecov](https://codecov.io/gh/stratum-mining/stratum/branch/main/graph/badge.svg?flag=extensions_sv2-coverage)](https://codecov.io/gh/stratum-mining/stratum)

The `extensions_sv2` crate provides message types and utilities for Stratum V2 protocol extensions. It includes support for Extensions Negotiation (0x0001), Worker-Specific Hashrate Tracking (0x0002) and Coinbase Transaction Payouts (0x0003), along with generic TLV (Type-Length-Value) encoding/decoding utilities that can be used by any extension requiring structured optional data fields.

## Usage
To include this crate in your project, run:

```bash
cargo add extensions_sv2
```

## Supported Extensions

- **Extensions Negotiation (0x0001)**: Negotiate which optional extensions are supported during connection setup
- **Worker-Specific Hashrate Tracking (0x0002)**: Track individual worker hashrates using TLV fields in `SubmitSharesExtended` messages
- **Coinbase Transaction Payouts (0x0003)**: Publish a weight-based payout distribution via `SetPayoutDistribution`, so the coinbase of a declared job pays multiple miners directly

For detailed specifications, see:
- [extensions-negotiation.md](https://github.com/stratum-mining/sv2-spec/blob/main/extensions/extensions-negotiation.md)
- [worker-specific-hashrate-tracking.md](https://github.com/stratum-mining/sv2-spec/blob/main/extensions/worker-specific-hashrate-tracking.md)
- [0x0003-coinbase-transaction-payouts.md](https://github.com/stratum-mining/sv2-spec/pull/203)