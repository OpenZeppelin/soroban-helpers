use sha2::{Digest, Sha256};
use stellar_rpc_client::{Client, Error, GetTransactionResponse};
use stellar_xdr::curr::{AccountEntry, Hash, TransactionEnvelope};

pub struct Provider {
    rpc_client: Client,
    network_passphrase: String,
    network_id: Hash,
}

impl Provider {
    pub fn new(rpc_url: &str, network_passphrase: &str) -> Result<Self, Error> {
        let rpc_client = Client::new(rpc_url)?;
        let network_id = Hash(Sha256::digest(network_passphrase.as_bytes()).into());

        Ok(Self {
            rpc_client,
            network_passphrase: network_passphrase.to_string(),
            network_id,
        })
    }

    pub fn network_passphrase(&self) -> &str {
        &self.network_passphrase
    }

    pub fn network_id(&self) -> &Hash {
        &self.network_id
    }

    pub async fn get_account(&self, account_id: &str) -> Result<AccountEntry, Error> {
        self.rpc_client.get_account(account_id).await
    }

    pub async fn simulate_transaction(
        &self,
        tx_envelope: &TransactionEnvelope,
    ) -> Result<stellar_rpc_client::SimulateTransactionResponse, Error> {
        self.rpc_client
            .simulate_transaction_envelope(tx_envelope)
            .await
    }

    pub async fn send_transaction(
        &self,
        tx_envelope: &TransactionEnvelope,
    ) -> Result<GetTransactionResponse, Error> {
        self.rpc_client.send_transaction_polling(tx_envelope).await
    }
}
