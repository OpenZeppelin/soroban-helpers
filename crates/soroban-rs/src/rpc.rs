//! # Soroban RPC Client
//!
//! This module provides the RPC client implementation for communicating with Soroban RPC servers.
//! It defines a trait for RPC operations and provides a concrete implementation using
//! the official Stellar RPC client.
//!
use crate::error::SorobanHelperError;
use stellar_rpc_client::Client;
use stellar_rpc_client::{GetTransactionResponse, SimulateTransactionResponse};
use stellar_xdr::curr::{AccountEntry, TransactionEnvelope};

/// Interface for RPC operations with Soroban servers.
///
/// This trait defines the core operations that any Soroban RPC client must implement,
/// providing an abstraction layer that allows for different implementations,
/// including mock implementations for testing.
#[async_trait::async_trait]
pub trait RpcClient: Send + Sync {
    async fn get_account(&self, account_id: &str) -> Result<AccountEntry, SorobanHelperError>;
    async fn simulate_transaction_envelope(
        &self,
        tx_envelope: &TransactionEnvelope,
    ) -> Result<SimulateTransactionResponse, SorobanHelperError>;
    async fn send_transaction_polling(
        &self,
        tx_envelope: &TransactionEnvelope,
    ) -> Result<GetTransactionResponse, SorobanHelperError>;
}

/// Implementation of the RPC client using the official Stellar RPC client.
///
/// This client connects to a Soroban RPC server endpoint and provides
/// methods for interacting with the Stellar network.
pub struct ExternalRpcClient {
    /// The internal Stellar RPC client
    client: Client,
}

impl ExternalRpcClient {
    /// Creates a new external RPC client connected to the specified URL.
    ///
    /// # Parameters
    ///
    /// * `url` - The URL of the Soroban RPC server
    ///
    /// # Returns
    ///
    /// A new `ExternalRpcClient` or an error if the client could not be created
    ///
    /// # Errors
    ///
    /// Returns `SorobanHelperError::NetworkRequestFailed` if the client initialization fails
    pub fn new(url: &str) -> Result<Self, SorobanHelperError> {
        let client = Client::new(url).map_err(|e| {
            SorobanHelperError::NetworkRequestFailed(format!("Failed to create client: {}", e))
        })?;
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl RpcClient for ExternalRpcClient {
    /// Retrieves account information from the Stellar network.
    ///
    /// # Parameters
    ///
    /// * `account_id` - The Stellar account ID to retrieve
    ///
    /// # Returns
    ///
    /// The account entry information or an error if the account could not be retrieved
    async fn get_account(&self, account_id: &str) -> Result<AccountEntry, SorobanHelperError> {
        self.client
            .get_account(account_id)
            .await
            .map_err(|e| SorobanHelperError::NetworkRequestFailed(format!("Error: {}", e)))
    }

    /// Simulates a transaction without submitting it to the network.
    ///
    /// # Parameters
    ///
    /// * `tx_envelope` - The transaction envelope to simulate
    ///
    /// # Returns
    ///
    /// The simulation response or an error if the simulation failed
    async fn simulate_transaction_envelope(
        &self,
        tx_envelope: &TransactionEnvelope,
    ) -> Result<SimulateTransactionResponse, SorobanHelperError> {
        self.client
            .simulate_transaction_envelope(tx_envelope)
            .await
            .map_err(|e| SorobanHelperError::NetworkRequestFailed(format!("Error: {}", e)))
    }

    /// Submits a transaction to the network and polls until completion.
    ///
    /// # Parameters
    ///
    /// * `tx_envelope` - The signed transaction envelope to submit
    ///
    /// # Returns
    ///
    /// The transaction response or an error if the transaction failed
    async fn send_transaction_polling(
        &self,
        tx_envelope: &TransactionEnvelope,
    ) -> Result<GetTransactionResponse, SorobanHelperError> {
        self.client
            .send_transaction_polling(tx_envelope)
            .await
            .map_err(|e| SorobanHelperError::NetworkRequestFailed(format!("Error: {}", e)))
    }
}

#[cfg(test)]
pub mod test {
    use crate::mock::{mock_signer1, mock_transaction_envelope};

    use super::*;

    #[test]
    fn test_new() {
        // use 443 port for https
        let client = ExternalRpcClient::new("https://test.com").unwrap();
        assert_eq!(client.client.base_url(), "https://test.com:443/");

        // use 80 port for http
        let client = ExternalRpcClient::new("http://test.com").unwrap();
        assert_eq!(client.client.base_url(), "http://test.com:80/");
    }

    #[tokio::test]
    async fn test_get_account_error() {
        let client = ExternalRpcClient::new("https://test.com").unwrap();
        let account_id = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let res = client.get_account(account_id).await;
        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            SorobanHelperError::NetworkRequestFailed(_)
        ));
    }

    #[tokio::test]
    async fn test_simulate_transaction_envelope_error() {
        let client = ExternalRpcClient::new("https://soroban-testnet.stellar.org").unwrap();
        let account_id = mock_signer1().account_id();
        let transaction_envelope = mock_transaction_envelope(account_id);
        let res = client
            .simulate_transaction_envelope(&transaction_envelope)
            .await;

        // simulations always succeed
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap().error.unwrap(),
            "Transaction contains more than one operation"
        );
    }

    #[tokio::test]
    async fn test_send_transaction_polling_error() {
        let client = ExternalRpcClient::new("https://soroban-testnet.stellar.org").unwrap();
        let account_id = mock_signer1().account_id();
        let transaction_envelope = mock_transaction_envelope(account_id);
        let res = client.send_transaction_polling(&transaction_envelope).await;
        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            SorobanHelperError::NetworkRequestFailed(_)
        ));
    }
}
