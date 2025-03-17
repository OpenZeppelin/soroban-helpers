use crate::{error::SorobanHelperError, rpc::{ExternalRpcClient, RpcClient}};
use sha2::{Digest, Sha256};
use stellar_rpc_client::{GetTransactionResponse, SimulateTransactionResponse};
use stellar_xdr::curr::{AccountEntry, Hash, TransactionEnvelope};
use std::sync::Arc;

#[derive(Clone)]
pub struct EnvConfigs {
    pub rpc_url: String,
    pub network_passphrase: String,
}

#[derive(Clone)]
pub struct Env {
    pub(crate) rpc_client: Arc<dyn RpcClient + Send + Sync>,
    pub(crate) configs: EnvConfigs,
}

impl Env {
    pub fn new(configs: EnvConfigs) -> Result<Self, SorobanHelperError> {
        let client = ExternalRpcClient::new(&configs.rpc_url)?;
        Ok(Self {
            rpc_client: Arc::new(client),
            configs,
        })
    }

    pub fn network_passphrase(&self) -> &str {
        &self.configs.network_passphrase
    }

    pub fn network_id(&self) -> Hash {
        let network_pass_bytes = self.configs.network_passphrase.as_bytes();
        Hash(Sha256::digest(network_pass_bytes).into())
    }

    pub async fn get_account(&self, account_id: &str) -> Result<AccountEntry, SorobanHelperError> {
        self.rpc_client.get_account(account_id).await.map_err(|e| {
            SorobanHelperError::NetworkRequestFailed(format!(
                "Failed to get account {}: {}",
                account_id, e
            ))
        })
    }

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

    pub async fn send_transaction(
        &self,
        tx_envelope: &TransactionEnvelope,
    ) -> Result<GetTransactionResponse, SorobanHelperError> {
        self.rpc_client
            .send_transaction_polling(tx_envelope)
            .await
            .map_err(|e| {
                // Check if this is a "contract code already exists" error
                let error_string = e.to_string();
                if error_string.contains("ContractCodeAlreadyExists") {
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
