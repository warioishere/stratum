//! Coinbase Transaction Payouts (extension_type=0x0003)
//!
//! This extension lets a Pool publish a weight-based payout distribution to Job Declarator
//! Clients, so that the coinbase of a declared job pays multiple miners of the Pool directly.
//!
//! JDS publishes the distribution via [`SetPayoutDistribution`]. Once the template revenue is
//! known, both sides derive the same output amounts from it via [`compute_payout_amounts`]: JDC
//! to build the coinbase, JDS/Pool to validate the declared one. JDC refers back to the
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

/// Coinbase output amounts derived from a payout distribution and a template revenue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayoutAmounts {
    /// Satoshis paid to `pool_payout`.
    pub pool_payout: u64,
    /// Satoshis paid to each entry of `payouts`, at the same index, or `None` when the entry is
    /// dust-pruned and gets no output.
    pub payouts: Vec<Option<u64>>,
}

/// Reasons a payout distribution cannot be applied to a template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayoutComputationError {
    /// `dust_limits` and `payouts` have different lengths.
    DustLimitsLengthMismatch,
    /// The Pool weight or a miner weight is 0.
    ZeroWeight,
    /// The fixed amounts of `additional_outputs` add up to at least the template revenue.
    FixedOutputsExceedRevenue,
}

impl fmt::Display for PayoutComputationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PayoutComputationError::DustLimitsLengthMismatch => {
                write!(f, "dust_limits and payouts have different lengths")
            }
            PayoutComputationError::ZeroWeight => write!(f, "payout weights must be non-zero"),
            PayoutComputationError::FixedOutputsExceedRevenue => {
                write!(
                    f,
                    "additional_outputs amount to at least the template revenue"
                )
            }
        }
    }
}

/// Splits `template_revenue` across a payout distribution.
///
/// `pool_weight` and `weights` are the amount fields of `pool_payout` and `payouts`, and
/// `fixed_amounts` are the amount fields of `additional_outputs`. The fixed amounts are paid off
/// the top, every miner receives `floor(weight * remaining / total_weight)` unless that is below
/// its dust limit, and the Pool receives whatever is left, so the amounts always add up to
/// exactly `template_revenue`.
pub fn compute_payout_amounts(
    template_revenue: u64,
    pool_weight: u64,
    weights: &[u64],
    dust_limits: &[u32],
    fixed_amounts: &[u64],
) -> Result<PayoutAmounts, PayoutComputationError> {
    if weights.len() != dust_limits.len() {
        return Err(PayoutComputationError::DustLimitsLengthMismatch);
    }
    if pool_weight == 0 || weights.contains(&0) {
        return Err(PayoutComputationError::ZeroWeight);
    }

    // summed in u128, so that an overflow is impossible and an oversized fixed amount is caught
    // by the comparison below
    let fixed_total: u128 = fixed_amounts.iter().map(|&amount| amount as u128).sum();
    if fixed_total >= template_revenue as u128 {
        return Err(PayoutComputationError::FixedOutputsExceedRevenue);
    }
    let remaining = template_revenue as u128 - fixed_total;
    let total_weight: u128 =
        pool_weight as u128 + weights.iter().map(|&weight| weight as u128).sum::<u128>();

    let mut paid_to_miners: u128 = 0;
    let payouts = weights
        .iter()
        .zip(dust_limits)
        .map(|(&weight, &dust_limit)| {
            // a u64 weight times a revenue below 2^64 fits in u128, and the quotient is at most
            // `remaining`, which fits in u64
            let amount = (weight as u128 * remaining / total_weight) as u64;
            if amount < dust_limit as u64 {
                None
            } else {
                paid_to_miners += amount as u128;
                Some(amount)
            }
        })
        .collect();

    Ok(PayoutAmounts {
        pool_payout: (remaining - paid_to_miners) as u64,
        payouts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_amounts_add_up_to_template_revenue() {
        let amounts = compute_payout_amounts(1_000_003, 2, &[49, 49], &[0, 0], &[]).unwrap();
        assert_eq!(amounts.payouts, vec![Some(490_001), Some(490_001)]);
        // rounding remainders are absorbed by the Pool payout
        assert_eq!(amounts.pool_payout, 20_001);
        assert_eq!(amounts.pool_payout + 2 * 490_001, 1_000_003);
    }

    #[test]
    fn test_dust_pruned_amount_goes_to_pool() {
        let amounts = compute_payout_amounts(1_000, 10, &[989, 1], &[0, 2], &[]).unwrap();
        assert_eq!(amounts.payouts, vec![Some(989), None]);
        assert_eq!(amounts.pool_payout, 11);
    }

    #[test]
    fn test_amount_equal_to_dust_limit_is_paid() {
        let amounts = compute_payout_amounts(100, 1, &[1], &[50], &[]).unwrap();
        assert_eq!(amounts.payouts, vec![Some(50)]);
        assert_eq!(amounts.pool_payout, 50);
    }

    #[test]
    fn test_fixed_amounts_are_paid_off_the_top() {
        let amounts = compute_payout_amounts(1_100, 1, &[1], &[0], &[60, 40]).unwrap();
        assert_eq!(amounts.payouts, vec![Some(500)]);
        assert_eq!(amounts.pool_payout, 500);
    }

    #[test]
    fn test_fixed_amounts_reaching_revenue_are_rejected() {
        assert_eq!(
            compute_payout_amounts(100, 1, &[1], &[0], &[60, 40]),
            Err(PayoutComputationError::FixedOutputsExceedRevenue)
        );
        assert_eq!(
            compute_payout_amounts(100, 1, &[1], &[0], &[u64::MAX, u64::MAX]),
            Err(PayoutComputationError::FixedOutputsExceedRevenue)
        );
    }

    #[test]
    fn test_zero_weights_are_rejected() {
        assert_eq!(
            compute_payout_amounts(100, 0, &[1], &[0], &[]),
            Err(PayoutComputationError::ZeroWeight)
        );
        assert_eq!(
            compute_payout_amounts(100, 1, &[1, 0], &[0, 0], &[]),
            Err(PayoutComputationError::ZeroWeight)
        );
    }

    #[test]
    fn test_dust_limits_length_must_match_payouts() {
        assert_eq!(
            compute_payout_amounts(100, 1, &[1, 1], &[0], &[]),
            Err(PayoutComputationError::DustLimitsLengthMismatch)
        );
    }

    #[test]
    fn test_maximal_weights_do_not_overflow() {
        let revenue = 21_000_000 * 100_000_000;
        let amounts =
            compute_payout_amounts(revenue, u64::MAX, &[u64::MAX, u64::MAX], &[0, 0], &[]).unwrap();
        assert_eq!(amounts.payouts, vec![Some(revenue / 3), Some(revenue / 3)]);
        assert_eq!(amounts.pool_payout, revenue - 2 * (revenue / 3));
    }

    #[test]
    fn test_no_payouts_pays_everything_to_pool() {
        let amounts = compute_payout_amounts(5_000, 7, &[], &[], &[100]).unwrap();
        assert!(amounts.payouts.is_empty());
        assert_eq!(amounts.pool_payout, 4_900);
    }
}
