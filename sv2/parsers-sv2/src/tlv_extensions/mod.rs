//! TLV field implementations for known SV2 extensions.
//!
//! This module provides `TlvField` trait implementations for extension types
//! defined in the `extensions_sv2` crate, along with extension-specific error types.

mod error;

pub use error::{DistributionIdError, ExtensionError, UserIdentityError};

use super::{Tlv, TlvField};
use crate::ParserError;
use extensions_sv2::{
    DistributionId, UserIdentity, EXTENSION_TYPE_COINBASE_TRANSACTION_PAYOUTS,
    EXTENSION_TYPE_WORKER_HASHRATE_TRACKING, TLV_FIELD_TYPE_DISTRIBUTION_ID,
    TLV_FIELD_TYPE_USER_IDENTITY,
};

extern crate alloc;
use alloc::vec::Vec;

/// Maximum length for worker identifiers in bytes as per the spec.
const MAX_USER_IDENTITY_LENGTH: usize = 32;

/// Implementation of TlvField trait for UserIdentity.
///
/// This provides the standard interface for encoding/decoding UserIdentity
/// as a TLV field in the Worker-Specific Hashrate Tracking extension.
impl TlvField for UserIdentity {
    const EXTENSION_TYPE: u16 = EXTENSION_TYPE_WORKER_HASHRATE_TRACKING;
    const FIELD_TYPE: u8 = TLV_FIELD_TYPE_USER_IDENTITY;

    /// Decodes a TLV from raw bytes.
    fn from_bytes(bytes: &[u8]) -> Result<Tlv, ParserError> {
        // TlvError auto-converts to ParserError::TlvError
        Tlv::decode(bytes).map_err(Into::into)
    }

    /// Encodes this UserIdentity as raw TLV bytes.
    fn to_bytes(&self) -> Result<Vec<u8>, ParserError> {
        let tlv = self.to_tlv()?;
        // TlvError auto-converts to ParserError::TlvError
        tlv.encode().map_err(Into::into)
    }

    /// Creates a UserIdentity from a parsed TLV.
    fn from_tlv(tlv: &Tlv) -> Result<Self, ParserError> {
        // Verify extension type
        if tlv.r#type.extension_type != Self::EXTENSION_TYPE {
            // UserIdentityError -> ExtensionError -> ParserError
            return Err(UserIdentityError::InvalidExtensionType(tlv.r#type.extension_type).into());
        }

        // Verify field type
        if tlv.r#type.field_type != Self::FIELD_TYPE {
            return Err(UserIdentityError::InvalidFieldType(tlv.r#type.field_type).into());
        }

        // Enforce 32-byte maximum
        if tlv.value.len() > MAX_USER_IDENTITY_LENGTH {
            return Err(UserIdentityError::TooLong(tlv.value.len()).into());
        }

        // Create UserIdentity from raw bytes
        UserIdentity::from_bytes(&tlv.value)
            .map_err(|e| UserIdentityError::InvalidUtf8(e.into()).into())
    }

    /// Converts this UserIdentity into a TLV structure.
    fn to_tlv(&self) -> Result<Tlv, ParserError> {
        // Validate length
        if self.len() > MAX_USER_IDENTITY_LENGTH {
            return Err(UserIdentityError::TooLong(self.len()).into());
        }

        Ok(Tlv::new(
            Self::EXTENSION_TYPE,
            Self::FIELD_TYPE,
            self.as_bytes().to_vec(),
        ))
    }
}

/// Length in bytes of the `distribution_id` TLV value, a U64.
const DISTRIBUTION_ID_LENGTH: usize = 8;

/// Implementation of TlvField trait for DistributionId.
///
/// This provides the standard interface for encoding/decoding DistributionId
/// as a TLV field in the Coinbase Transaction Payouts extension.
impl TlvField for DistributionId {
    const EXTENSION_TYPE: u16 = EXTENSION_TYPE_COINBASE_TRANSACTION_PAYOUTS;
    const FIELD_TYPE: u8 = TLV_FIELD_TYPE_DISTRIBUTION_ID;

    /// Decodes a TLV from raw bytes.
    fn from_bytes(bytes: &[u8]) -> Result<Tlv, ParserError> {
        Tlv::decode(bytes).map_err(Into::into)
    }

    /// Encodes this DistributionId as raw TLV bytes.
    fn to_bytes(&self) -> Result<Vec<u8>, ParserError> {
        let tlv = self.to_tlv()?;
        tlv.encode().map_err(Into::into)
    }

    /// Creates a DistributionId from a parsed TLV.
    fn from_tlv(tlv: &Tlv) -> Result<Self, ParserError> {
        if tlv.r#type.extension_type != Self::EXTENSION_TYPE {
            return Err(
                DistributionIdError::InvalidExtensionType(tlv.r#type.extension_type).into(),
            );
        }

        if tlv.r#type.field_type != Self::FIELD_TYPE {
            return Err(DistributionIdError::InvalidFieldType(tlv.r#type.field_type).into());
        }

        let value: [u8; DISTRIBUTION_ID_LENGTH] = tlv
            .value
            .as_slice()
            .try_into()
            .map_err(|_| DistributionIdError::InvalidLength(tlv.value.len()))?;
        Ok(DistributionId(u64::from_le_bytes(value)))
    }

    /// Converts this DistributionId into a TLV structure.
    fn to_tlv(&self) -> Result<Tlv, ParserError> {
        Ok(Tlv::new(
            Self::EXTENSION_TYPE,
            Self::FIELD_TYPE,
            self.0.to_le_bytes().to_vec(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_distribution_id_wire_format() {
        let bytes = DistributionId(0x0102_0304_0506_0708).to_bytes().unwrap();
        assert_eq!(
            bytes,
            vec![0x03, 0x00, 0x01, 0x08, 0x00, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
        );
        let tlv = DistributionId::from_bytes(&bytes).unwrap();
        assert_eq!(
            DistributionId::from_tlv(&tlv).unwrap(),
            DistributionId(0x0102_0304_0506_0708)
        );
    }

    #[test]
    fn test_distribution_id_rejects_wrong_length() {
        let tlv = Tlv::new(
            EXTENSION_TYPE_COINBASE_TRANSACTION_PAYOUTS,
            TLV_FIELD_TYPE_DISTRIBUTION_ID,
            vec![1, 2, 3, 4],
        );
        assert!(matches!(
            DistributionId::from_tlv(&tlv),
            Err(ParserError::ExtensionError(ExtensionError::DistributionId(
                DistributionIdError::InvalidLength(4)
            )))
        ));
    }

    #[test]
    fn test_distribution_id_rejects_other_field() {
        let tlv = Tlv::new(
            EXTENSION_TYPE_WORKER_HASHRATE_TRACKING,
            TLV_FIELD_TYPE_DISTRIBUTION_ID,
            vec![0; DISTRIBUTION_ID_LENGTH],
        );
        assert!(matches!(
            DistributionId::from_tlv(&tlv),
            Err(ParserError::ExtensionError(ExtensionError::DistributionId(
                DistributionIdError::InvalidExtensionType(EXTENSION_TYPE_WORKER_HASHRATE_TRACKING)
            )))
        ));
    }
}
