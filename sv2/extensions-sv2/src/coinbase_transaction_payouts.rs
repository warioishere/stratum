//! Coinbase Transaction Payouts (extension_type=0x0003)
//!
//! This extension lets a Pool publish a weight-based payout distribution to Job Declarator
//! Clients, so that the coinbase of a declared job pays multiple miners of the Pool directly.
//!
//! JDS publishes the distribution via [`SetPayoutDistribution`], and JDC refers back to the
//! distribution a job was built from via the [`DistributionId`] TLV field.

use alloc::{fmt, vec::Vec};
use binary_sv2::{Deserialize, Seq064K, Serialize, B064K};

/// Extension type for Coinbase Transaction Payouts
pub const EXTENSION_TYPE: u16 = 0x0003;

/// Message type constants
pub const MESSAGE_TYPE_SET_PAYOUT_DISTRIBUTION: u8 = 0x00;

/// Channel message bits (all false for extensions as per spec)
pub const CHANNEL_BIT_SET_PAYOUT_DISTRIBUTION: bool = false;

/// TLV field type for `distribution_id` within this extension
pub const FIELD_TYPE_DISTRIBUTION_ID: u8 = 0x01;

/// Error code for `DeclareMiningJob.Error` and `SetCustomMiningJob.Error`: the declaration refers
/// to a `distribution_id` outside the accepted window (too old, or invalidated by a settlement).
pub const ERROR_CODE_STALE_PAYOUT_DISTRIBUTION: &str = "stale-payout-distribution";

/// Error code for `DeclareMiningJob.Error` and `SetCustomMiningJob.Error`: the declared coinbase
/// outputs do not match the ones computed from the referenced distribution.
pub const ERROR_CODE_INVALID_PAYOUT_DISTRIBUTION: &str = "invalid-payout-distribution";

/// Message used by JDS to publish the payout distribution that coinbase outputs of declared jobs
/// must follow.
///
/// It is the first message JDS sends after the extension is negotiated, and JDS sends a new one,
/// with a higher `distribution_id`, whenever the Pool updates the distribution.
///
/// All outputs are consensus-serialized. The amount fields of `pool_payout` and `payouts` encode
/// relative weights rather than satoshis, while the amount fields of `additional_outputs` encode
/// fixed satoshi amounts.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SetPayoutDistribution<'decoder> {
    /// Strictly increasing identifier, universal across all Pool/JDS connections of this Pool.
    pub distribution_id: u64,
    /// Output controlled by the Pool, whose amount field encodes the Pool's weight.
    pub pool_payout: B064K<'decoder>,
    /// Outputs paying miners, whose amount fields encode their weights.
    pub payouts: Seq064K<'decoder, B064K<'decoder>>,
    /// Dust limit in satoshis for each entry of `payouts`, at the same index.
    pub dust_limits: Seq064K<'decoder, u32>,
    /// Outputs added by the Pool, whose amount fields encode fixed satoshi amounts.
    pub additional_outputs: Seq064K<'decoder, B064K<'decoder>>,
}

impl fmt::Display for SetPayoutDistribution<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SetPayoutDistribution(distribution_id: {}, pool_payout: {}, payouts: {}, additional_outputs: {})",
            self.distribution_id,
            self.pool_payout,
            self.payouts.len(),
            self.additional_outputs.len()
        )
    }
}

impl fmt::Display for SetPayoutDistributionOwned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SetPayoutDistribution(distribution_id: {}, pool_payout: {}, payouts: {}, additional_outputs: {})",
            self.distribution_id,
            self.pool_payout,
            self.payouts.len(),
            self.additional_outputs.len()
        )
    }
}

/// The `distribution_id` TLV field.
///
/// JDC appends it to `DeclareMiningJob` (Full-Template mode) or `SetCustomMiningJob`
/// (Coinbase-only mode) to name the [`SetPayoutDistribution`] the job was built from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistributionId(pub u64);

impl fmt::Display for DistributionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DistributionId({})", self.0)
    }
}
