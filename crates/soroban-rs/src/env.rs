//! # Soroban Environment
//!
//! This module provides environment configuration and network interaction for Soroban operations.
//! The environment manages RPC connections, network configuration, and transaction handling.
//!
//! ## Features
//!
//! - RPC client configuration and management
//! - Network identification and parameters
//! - Account information retrieval
//! - Transaction simulation and submission
//!
//! ## Example
//!
//! ```rust
//! use soroban_rs::{Env, EnvConfigs};
//! use stellar_xdr::curr::TransactionEnvelope;
//!
//! async fn example() {
//!     // Create a new environment for the Stellar testnet
//!     let env = Env::new(EnvConfigs {
//!         rpc_url: "https://soroban-testnet.stellar.org".to_string(),
//!         network_passphrase: "Test SDF Network ; September 2015".to_string(),
//!     }).unwrap();
//!
//!     // Retrieve account information
//!     let account = env.get_account("G..........").await.unwrap();
//!     println!("Account: {:?}", account);
//! }
//! ```
use crate::{
    error::SorobanHelperError,
    rpc::{ExternalRpcClient, RpcClient},
    SorobanTransactionResponse,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use stellar_rpc_client::SimulateTransactionResponse;
use stellar_xdr::curr::{AccountEntry, Hash, TransactionEnvelope};

/// Configuration for a Soroban environment.
///
/// Contains the necessary parameters to connect to a Soroban RPC server
/// and identify the target network.
#[derive(Clone)]
pub struct EnvConfigs {
    /// URL of the Soroban RPC server
    pub rpc_url: String,
    /// Network passphrase that identifies the Stellar network
    pub network_passphrase: String,
}

/// The environment for Soroban operations.
///
/// Provides access to network functionality such as retrieving account information,
/// simulating transactions, and submitting transactions to the network.
#[derive(Clone)]
pub struct Env {
    /// RPC client for interacting with the Soroban network
    pub(crate) rpc_client: Arc<dyn RpcClient + Send + Sync>,
    /// Configuration for this environment
    pub(crate) configs: EnvConfigs,
}

impl Env {
    /// Creates a new environment with the specified configuration.
    ///
    /// # Parameters
    ///
    /// * `configs` - The environment configuration including RPC URL and network passphrase
    ///
    /// # Returns
    ///
    /// A new `Env` instance or an error if the RPC client could not be created
    ///
    /// # Errors
    ///
    /// Returns `SorobanHelperError` if the RPC client initialization fails
    pub fn new(configs: EnvConfigs) -> Result<Self, SorobanHelperError> {
        let client = ExternalRpcClient::new(&configs.rpc_url)?;
        Ok(Self {
            rpc_client: Arc::new(client),
            configs,
        })
    }

    /// Returns the network passphrase for this environment.
    ///
    /// The network passphrase is a string that uniquely identifies a Stellar network,
    /// such as "Public Global Stellar Network ; September 2015" for the public network
    /// or "Test SDF Network ; September 2015" for the testnet.
    ///
    /// # Returns
    ///
    /// The network passphrase as a string slice
    pub fn network_passphrase(&self) -> &str {
        &self.configs.network_passphrase
    }

    /// Calculates the network ID hash from the network passphrase.
    ///
    /// The network ID is the SHA-256 hash of the network passphrase and is used
    /// in various cryptographic operations, including transaction signing.
    ///
    /// # Returns
    ///
    /// The SHA-256 hash of the network passphrase
    pub fn network_id(&self) -> Hash {
        let network_pass_bytes = self.configs.network_passphrase.as_bytes();
        Hash(Sha256::digest(network_pass_bytes).into())
    }

    /// Retrieves account information from the network.
    ///
    /// # Parameters
    ///
    /// * `account_id` - The Stellar account ID to retrieve
    ///
    /// # Returns
    ///
    /// The account entry information or an error if the account could not be retrieved
    ///
    /// # Errors
    ///
    /// Returns `SorobanHelperError::NetworkRequestFailed` if the RPC request fails
    pub async fn get_account(&self, account_id: &str) -> Result<AccountEntry, SorobanHelperError> {
        self.rpc_client.get_account(account_id).await.map_err(|e| {
            SorobanHelperError::NetworkRequestFailed(format!(
                "Failed to get account {}: {}",
                account_id, e
            ))
        })
    }

    /// Simulates a transaction without submitting it to the network.
    ///
    /// This is useful for estimating transaction costs, validating transactions,
    /// and retrieving the expected results of contract invocations.
    ///
    /// # Parameters
    ///
    /// * `tx_envelope` - The transaction envelope to simulate
    ///
    /// # Returns
    ///
    /// The simulation response or an error if the simulation failed
    ///
    /// # Errors
    ///
    /// Returns `SorobanHelperError::NetworkRequestFailed` if the RPC request fails
    pub async fn simulate_transaction(
        &self,
        tx_envelope: &TransactionEnvelope,
    ) -> Result<SimulateTransactionResponse, SorobanHelperError> {
        self.rpc_client
            .simulate_transaction_envelope(tx_envelope)
            .await
            .map_err(|e| {
                SorobanHelperError::NetworkRequestFailed(format!(
                    "Failed to simulate transaction: {}",
                    e
                ))
            })
    }

    /// Submits a transaction to the network and waits for the result.
    ///
    /// # Parameters
    ///
    /// * `tx_envelope` - The signed transaction envelope to submit
    ///
    /// # Returns
    ///
    /// The transaction response or an error if the transaction failed
    ///
    /// # Errors
    ///
    /// Returns:
    /// - `SorobanHelperError::ContractCodeAlreadyExists` if the transaction failed because the contract code already exists
    /// - `SorobanHelperError::NetworkRequestFailed` for other transaction failures
    pub async fn send_transaction(
        &self,
        tx_envelope: &TransactionEnvelope,
    ) -> Result<SorobanTransactionResponse, SorobanHelperError> {
        self.rpc_client
            .send_transaction_polling(tx_envelope)
            .await
            .map_err(|e| {
                // Check if this is a "contract code already exists" error
                let error_string = e.to_string();
                if error_string.contains(&SorobanHelperError::ContractCodeAlreadyExists.to_string())
                {
                    return SorobanHelperError::ContractCodeAlreadyExists;
                }
                // Otherwise, it's a general transaction failure
                SorobanHelperError::NetworkRequestFailed(format!(
                    "Failed to send transaction: {}",
                    e
                ))
            })
    }
}

#[cfg(test)]
pub mod test {
    use crate::mock::{mock_env, mock_signer3, mock_transaction_envelope};

    use super::*;

    #[test]
    fn test_new() {
        let env = Env::new(EnvConfigs {
            rpc_url: "https://soroban-testnet.stellar.org".to_string(),
            network_passphrase: "Test SDF Network ; September 2015".to_string(),
        })
        .unwrap();

        assert_eq!(env.configs.rpc_url, "https://soroban-testnet.stellar.org");
        assert_eq!(
            env.configs.network_passphrase,
            "Test SDF Network ; September 2015"
        );
    }

    #[test]
    fn test_network_id() {
        let env = Env::new(EnvConfigs {
            rpc_url: "https://test.com".to_string(),
            network_passphrase: "test".to_string(),
        })
        .unwrap();

        assert_eq!(
            env.network_id().0,
            [
                159, 134, 208, 129, 136, 76, 125, 101, 154, 47, 234, 160, 197, 90, 208, 21, 163,
                191, 79, 27, 43, 11, 130, 44, 209, 93, 108, 21, 176, 240, 10, 8
            ]
        );
    }

    #[tokio::test]
    async fn test_code_already_exists_error() {
        let send_transaction_polling_result = Err(SorobanHelperError::ContractCodeAlreadyExists);
        let env = mock_env(None, None, Some(send_transaction_polling_result));
        let account_id = mock_signer3().account_id();
        let result = env
            .send_transaction(&mock_transaction_envelope(account_id))
            .await;
        assert!(matches!(
            result,
            Err(SorobanHelperError::ContractCodeAlreadyExists)
        ));
    }

    #[tokio::test]
    async fn test_send_transaction_error() {
        let send_transaction_polling_result = Err(SorobanHelperError::NetworkRequestFailed(
            "OtherError".to_string(),
        ));
        let env = mock_env(None, None, Some(send_transaction_polling_result));
        let account_id = mock_signer3().account_id();
        let result = env
            .send_transaction(&mock_transaction_envelope(account_id))
            .await;
        assert!(matches!(
            result,
            Err(SorobanHelperError::NetworkRequestFailed(_))
        ));
    }
}
