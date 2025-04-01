//! # Soroban Error Handling
//!
//! This module defines the error types used throughout the Soroban helpers library.
//! It provides a unified error handling approach for all operations related to
//! Soroban contract deployment, invocation, and transaction management.
use std::{error::Error, fmt};

/// Errors that can occur when using the Soroban helpers library.
///
/// This enum covers errors from various operations including transaction
/// submission, signing, contract deployment, and network communication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SorobanHelperError {
    /// Error when a transaction fails to execute successfully.
    TransactionFailed(String),

    /// Error when a transaction simulation fails.
    TransactionSimulationFailed(String),

    /// Error when attempting to upload contract code that already exists.
    ContractCodeAlreadyExists,

    /// Error when a network request to the Soroban RPC server fails.
    NetworkRequestFailed(String),

    /// Error when a signing operation fails.
    SigningFailed(String),

    /// Error when XDR encoding or decoding fails.
    XdrEncodingFailed(String),

    /// Error when an invalid argument is provided to a function.
    InvalidArgument(String),

    /// Error when building a transaction fails.
    TransactionBuildFailed(String),

    /// Error when an operation requires authorization that isn't present.
    Unauthorized(String),

    /// Error when attempting to invoke a contract without setting deployment configs.
    ContractDeployedConfigsNotSet,

    /// Error when a file operation fails.
    FileReadError(String),

    /// Error when a conversion fails.
    ConversionError(String),

    // Some client operations taht it's still not supported
    NotSupported(String),
}

impl fmt::Display for SorobanHelperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TransactionFailed(msg) => write!(f, "Transaction failed: {}", msg),
            Self::TransactionSimulationFailed(msg) => {
                write!(f, "Transaction simulation failed: {}", msg)
            }
            Self::ContractCodeAlreadyExists => write!(f, "Contract code already exists"),
            Self::NetworkRequestFailed(msg) => write!(f, "Network request failed: {}", msg),
            Self::SigningFailed(msg) => write!(f, "Signing operation failed: {}", msg),
            Self::XdrEncodingFailed(msg) => write!(f, "XDR encoding failed: {}", msg),
            Self::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            Self::TransactionBuildFailed(msg) => write!(f, "Transaction build failed: {}", msg),
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            Self::ContractDeployedConfigsNotSet => write!(f, "Contract deployed configs not set"),
            Self::FileReadError(msg) => write!(f, "File read error: {}", msg),
            Self::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            Self::NotSupported(msg) => write!(f, "Not supported: {}", msg),
        }
    }
}

impl Error for SorobanHelperError {}

/// Convert XDR errors into SorobanHelperError
impl From<stellar_xdr::curr::Error> for SorobanHelperError {
    fn from(err: stellar_xdr::curr::Error) -> Self {
        Self::XdrEncodingFailed(err.to_string())
    }
}

/// Convert IO errors into SorobanHelperError
impl From<std::io::Error> for SorobanHelperError {
    fn from(err: std::io::Error) -> Self {
        Self::InvalidArgument(format!("File operation failed: {}", err))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_display_implementations() {
        let cases = [
            (
                SorobanHelperError::TransactionFailed("timeout".to_string()),
                "Transaction failed: timeout",
            ),
            (
                SorobanHelperError::ContractCodeAlreadyExists,
                "Contract code already exists",
            ),
            (
                SorobanHelperError::NetworkRequestFailed("connection refused".to_string()),
                "Network request failed: connection refused",
            ),
            (
                SorobanHelperError::SigningFailed("invalid key".to_string()),
                "Signing operation failed: invalid key",
            ),
            (
                SorobanHelperError::XdrEncodingFailed("invalid format".to_string()),
                "XDR encoding failed: invalid format",
            ),
            (
                SorobanHelperError::InvalidArgument("wrong type".to_string()),
                "Invalid argument: wrong type",
            ),
            (
                SorobanHelperError::TransactionBuildFailed("missing field".to_string()),
                "Transaction build failed: missing field",
            ),
            (
                SorobanHelperError::Unauthorized("missing signature".to_string()),
                "Unauthorized: missing signature",
            ),
            (
                SorobanHelperError::ContractDeployedConfigsNotSet,
                "Contract deployed configs not set",
            ),
            (
                SorobanHelperError::FileReadError("file not found".to_string()),
                "File read error: file not found",
            ),
            (
                SorobanHelperError::ConversionError("invalid type conversion".to_string()),
                "Conversion error: invalid type conversion",
            ),
            (
                SorobanHelperError::TransactionSimulationFailed("bad input".to_string()),
                "Transaction simulation failed: bad input",
            ),
            (
                SorobanHelperError::NotSupported("feature not implemented".to_string()),
                "Not supported: feature not implemented",
            ),
        ];

        for (error, expected_msg) in cases {
            assert_eq!(error.to_string(), expected_msg);
        }
    }

    #[test]
    fn test_from_io_error() {
        let io_error = IoError::new(ErrorKind::NotFound, "file not found");
        let helper_error = SorobanHelperError::from(io_error);

        assert!(
            matches!(helper_error, SorobanHelperError::InvalidArgument(_)),
            "Expected InvalidArgument variant"
        );

        let error_string = helper_error.to_string();
        assert!(error_string.contains("file not found"));
        assert!(error_string.contains("File operation failed"));
    }

    #[test]
    fn test_from_xdr_error() {
        // Create a mock XDR error
        let xdr_error = stellar_xdr::curr::Error::Invalid;
        let helper_error = SorobanHelperError::from(xdr_error);

        assert!(
            matches!(helper_error, SorobanHelperError::XdrEncodingFailed(_)),
            "Expected XdrEncodingFailed variant"
        );

        let error_string = helper_error.to_string();
        assert!(error_string.contains("XDR encoding failed"));
    }

    #[test]
    fn test_error_trait_implementation() {
        // Test that SorobanHelperError implements the Error trait correctly
        let error = SorobanHelperError::InvalidArgument("test error".to_string());
        let _: &dyn Error = &error; // This will fail to compile if Error is not implemented

        // Test Debug implementation
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidArgument"));
    }

    #[test]
    fn test_error_equality() {
        // Test PartialEq implementation
        let error1 = SorobanHelperError::InvalidArgument("test error".to_string());
        let error2 = SorobanHelperError::InvalidArgument("test error".to_string());
        let error3 = SorobanHelperError::InvalidArgument("different error".to_string());

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);

        let different_type = SorobanHelperError::NetworkRequestFailed("test error".to_string());
        assert_ne!(error1, different_type);
    }

    #[test]
    fn test_error_cloning() {
        // Test Clone implementation
        let original = SorobanHelperError::TransactionFailed("test failure".to_string());
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }
}
