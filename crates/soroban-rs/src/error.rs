use std::{error::Error, fmt};

#[derive(Debug)]
pub enum SorobanHelperError {
    TransactionFailed(String),
    ContractCodeAlreadyExists,
    NetworkRequestFailed(String),
    SigningFailed(String),
    XdrEncodingFailed(String),
    InvalidArgument(String),
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
        }
    }
}

impl Error for SorobanHelperError {}

impl From<stellar_xdr::curr::Error> for SorobanHelperError {
    fn from(err: stellar_xdr::curr::Error) -> Self {
        Self::XdrEncodingFailed(err.to_string())
    }
}

impl From<std::io::Error> for SorobanHelperError {
    fn from(err: std::io::Error) -> Self {
        Self::InvalidArgument(format!("File operation failed: {}", err))
    }
}
