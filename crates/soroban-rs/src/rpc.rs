use crate::error::SorobanHelperError;
use stellar_rpc_client::Client;
use stellar_rpc_client::{GetTransactionResponse, SimulateTransactionResponse};
use stellar_xdr::curr::{AccountEntry, TransactionEnvelope};

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

pub struct ExternalRpcClient {
    inner: Client,
}

impl ExternalRpcClient {
    pub fn new(url: &str) -> Result<Self, SorobanHelperError> {
        let client = Client::new(url).map_err(|e| {
            SorobanHelperError::NetworkRequestFailed(format!("Failed to create client: {}", e))
        })?;
        Ok(Self { inner: client })
    }
}

#[async_trait::async_trait]
impl RpcClient for ExternalRpcClient {
    async fn get_account(&self, account_id: &str) -> Result<AccountEntry, SorobanHelperError> {
        self.inner
            .get_account(account_id)
            .await
            .map_err(|e| SorobanHelperError::NetworkRequestFailed(format!("Error: {}", e)))
    }

    async fn simulate_transaction_envelope(
        &self,
        tx_envelope: &TransactionEnvelope,
    ) -> Result<SimulateTransactionResponse, SorobanHelperError> {
        self.inner
            .simulate_transaction_envelope(tx_envelope)
            .await
            .map_err(|e| SorobanHelperError::NetworkRequestFailed(format!("Error: {}", e)))
    }

    async fn send_transaction_polling(
        &self,
        tx_envelope: &TransactionEnvelope,
    ) -> Result<GetTransactionResponse, SorobanHelperError> {
        self.inner
            .send_transaction_polling(tx_envelope)
            .await
            .map_err(|e| SorobanHelperError::NetworkRequestFailed(format!("Error: {}", e)))
    }
}
