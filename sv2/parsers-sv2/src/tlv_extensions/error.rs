use alloc::string::String;
use core::fmt;

/// Errors that can occur when working with TLV extension fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionError {
    /// Worker-Specific Hashrate Tracking extension error.
    UserIdentity(UserIdentityError),
    /// Coinbase Transaction Payouts extension error.
    DistributionId(DistributionIdError),
}

impl fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtensionError::UserIdentity(e) => write!(f, "UserIdentity error: {}", e),
            ExtensionError::DistributionId(e) => write!(f, "DistributionId error: {}", e),
        }
    }
}

impl From<UserIdentityError> for ExtensionError {
    fn from(e: UserIdentityError) -> Self {
        ExtensionError::UserIdentity(e)
    }
}

impl From<UserIdentityError> for crate::ParserError {
    fn from(e: UserIdentityError) -> Self {
        crate::ParserError::ExtensionError(ExtensionError::UserIdentity(e))
    }
}

impl From<DistributionIdError> for ExtensionError {
    fn from(e: DistributionIdError) -> Self {
        ExtensionError::DistributionId(e)
    }
}

impl From<DistributionIdError> for crate::ParserError {
    fn from(e: DistributionIdError) -> Self {
        crate::ParserError::ExtensionError(ExtensionError::DistributionId(e))
    }
}

/// Errors specific to the Coinbase Transaction Payouts extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistributionIdError {
    /// Extension type in TLV does not match expected value (0x0003).
    ///
    /// Contains the actual extension_type found.
    InvalidExtensionType(u16),

    /// Field type in TLV does not match expected value (0x01).
    ///
    /// Contains the actual field_type found.
    InvalidFieldType(u8),

    /// TLV value is not exactly 8 bytes long.
    ///
    /// Contains the actual length in bytes.
    InvalidLength(usize),
}

impl fmt::Display for DistributionIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DistributionIdError::InvalidExtensionType(ext_type) => write!(
                f,
                "Invalid extension type 0x{:04x}, expected 0x0003 (Coinbase Transaction Payouts)",
                ext_type
            ),
            DistributionIdError::InvalidFieldType(field_type) => write!(
                f,
                "Invalid field type 0x{:02x}, expected 0x01 (DistributionId)",
                field_type
            ),
            DistributionIdError::InvalidLength(len) => write!(
                f,
                "DistributionId length {} bytes, expected exactly 8 bytes",
                len
            ),
        }
    }
}

/// Errors specific to the Worker-Specific Hashrate Tracking extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserIdentityError {
    /// UserIdentity exceeds the maximum allowed length of 32 bytes.
    ///
    /// Contains the actual length in bytes.
    TooLong(usize),

    /// Extension type in TLV does not match expected value (0x0002).
    ///
    /// Contains the actual extension_type found.
    InvalidExtensionType(u16),

    /// Field type in TLV does not match expected value (0x01).
    ///
    /// Contains the actual field_type found.
    InvalidFieldType(u8),

    /// Invalid UTF-8 in UserIdentity.
    InvalidUtf8(String),
}

impl fmt::Display for UserIdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserIdentityError::TooLong(len) => write!(
                f,
                "UserIdentity length {} bytes exceeds maximum of 32 bytes",
                len
            ),
            UserIdentityError::InvalidExtensionType(ext_type) => write!(
                f,
                "Invalid extension type 0x{:04x}, expected 0x0002 (Worker-Specific Hashrate Tracking)",
                ext_type
            ),
            UserIdentityError::InvalidFieldType(field_type) => write!(
                f,
                "Invalid field type 0x{:02x}, expected 0x01 (UserIdentity)",
                field_type
            ),
            UserIdentityError::InvalidUtf8(msg) => write!(f, "Invalid UTF-8: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_user_identity_error_display() {
        let err = UserIdentityError::TooLong(50);
        assert!(err.to_string().contains("50"));
        assert!(err.to_string().contains("32"));

        let err = UserIdentityError::InvalidExtensionType(0x0099);
        assert!(err.to_string().contains("0x0099"));
        assert!(err.to_string().contains("0x0002"));

        let err = UserIdentityError::InvalidFieldType(0x05);
        assert!(err.to_string().contains("0x05"));
        assert!(err.to_string().contains("0x01"));
    }

    #[test]
    fn test_extension_error_from_user_identity_error() {
        let user_err = UserIdentityError::TooLong(50);
        let ext_err: ExtensionError = user_err.into();
        assert!(matches!(ext_err, ExtensionError::UserIdentity(_)));
    }
}
