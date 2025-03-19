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
#[derive(Debug)]
pub enum SorobanHelperError {
    /// Error when a transaction fails to execute successfully.
    TransactionFailed(String),

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
}

impl fmt::Display for SorobanHelperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TransactionFailed(msg) => write!(f, "Transaction failed: {}", msg),
            Self::ContractCodeAlreadyExists => write!(f, "Contract code already exists"),
            Self::NetworkRequestFailed(msg) => write!(f, "Network request failed: {}", msg),
            Self::SigningFailed(msg) => write!(f, "Signing operation failed: {}", msg),
            Self::XdrEncodingFailed(msg) => write!(f, "XDR encoding failed: {}", msg),
            Self::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            Self::TransactionBuildFailed(msg) => write!(f, "Transaction build failed: {}", msg),
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            Self::ContractDeployedConfigsNotSet => write!(f, "Contract deployed configs not set"),
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
