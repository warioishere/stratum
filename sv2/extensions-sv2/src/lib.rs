//! # Stratum V2 Extensions Messages Crate.
//!
//! This crate defines extension messages for Stratum V2 protocol.
//!
//! ## Extensions Supported
//!
//! - **Extensions Negotiation** (extension_type=0x0001): Allows endpoints to negotiate
//!   which optional extensions are supported during connection setup.
//! - **Worker-Specific Hashrate Tracking** (extension_type=0x0002): Enables tracking per-worker hashrates
//!   within extended channels via TLV fields.
//! - **Coinbase Transaction Payouts** (extension_type=0x0003): Lets the coinbase of a declared job
//!   pay multiple miners of the Pool directly, according to a weight-based payout distribution.
//!
//! ## Architecture
//!
//! The crate is organized into:
//! - `extensions_negotiation`: Extension negotiation protocol
//! - `worker_specific_hashrate_tracking`: Worker-Specific Hashrate Tracking extension
//! - `coinbase_transaction_payouts`: Coinbase Transaction Payouts extension
//!
//! TLV encoding/decoding utilities are provided by the `parsers_sv2` crate.
//!
//! For further information about the extensions, please refer to:
//! - [Extensions Negotiation Spec](https://github.com/stratum-mining/sv2-spec/blob/main/extensions/extensions-negotiation.md)
//! - [Worker-Specific Hashrate Tracking Spec](https://github.com/stratum-mining/sv2-spec/blob/main/extensions/worker-specific-hashrate-tracking.md)
//! - [Coinbase Transaction Payouts Spec](https://github.com/stratum-mining/sv2-spec/pull/203)

#![no_std]

extern crate alloc;

// Extensions Negotiation (0x0001)
pub mod extensions_negotiation;

// Worker-Specific Hashrate Tracking (0x0002)
pub mod worker_specific_hashrate_tracking;

// Coinbase Transaction Payouts (0x0003)
pub mod coinbase_transaction_payouts;

// Re-export commonly used items from extensions_negotiation
pub use extensions_negotiation::{
    RequestExtensions, RequestExtensionsError, RequestExtensionsErrorOwned, RequestExtensionsOwned,
    RequestExtensionsSuccess, RequestExtensionsSuccessOwned, CHANNEL_BIT_REQUEST_EXTENSIONS,
    CHANNEL_BIT_REQUEST_EXTENSIONS_ERROR, CHANNEL_BIT_REQUEST_EXTENSIONS_SUCCESS,
    EXTENSION_TYPE as EXTENSION_TYPE_EXTENSIONS_NEGOTIATION, MESSAGE_TYPE_REQUEST_EXTENSIONS,
    MESSAGE_TYPE_REQUEST_EXTENSIONS_ERROR, MESSAGE_TYPE_REQUEST_EXTENSIONS_SUCCESS,
};

// Re-export commonly used items from worker_specific_hashrate_tracking
pub use worker_specific_hashrate_tracking::{
    UserIdentity, EXTENSION_TYPE as EXTENSION_TYPE_WORKER_HASHRATE_TRACKING,
    FIELD_TYPE_USER_IDENTITY as TLV_FIELD_TYPE_USER_IDENTITY, MAX_USER_IDENTITY_LENGTH,
};

// Re-export commonly used items from coinbase_transaction_payouts
pub use coinbase_transaction_payouts::{
    DistributionId, SetPayoutDistribution, SetPayoutDistributionOwned,
    CHANNEL_BIT_SET_PAYOUT_DISTRIBUTION, ERROR_CODE_INVALID_PAYOUT_DISTRIBUTION,
    ERROR_CODE_STALE_PAYOUT_DISTRIBUTION,
    EXTENSION_TYPE as EXTENSION_TYPE_COINBASE_TRANSACTION_PAYOUTS,
    FIELD_TYPE_DISTRIBUTION_ID as TLV_FIELD_TYPE_DISTRIBUTION_ID,
    MESSAGE_TYPE_SET_PAYOUT_DISTRIBUTION,
};
