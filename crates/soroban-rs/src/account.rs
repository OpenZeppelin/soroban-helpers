use crate::{Provider, Signer, error::SorobanHelperError};

pub struct AccountManager<'a> {
    provider: &'a Provider,
    signer: &'a Signer,
}

impl<'a> AccountManager<'a> {
    pub fn new(provider: &'a Provider, signer: &'a Signer) -> Self {
        Self { provider, signer }
    }

    pub async fn get_sequence(&self) -> Result<i64, SorobanHelperError> {
        let account_id = self.signer.account_id().clone();
        let account_details = self.provider.get_account(&account_id.to_string()).await?;
        Ok(account_details.seq_num.into())
    }

    pub fn account_id(&self) -> stellar_xdr::curr::AccountId {
        self.signer.account_id()
    }

    pub fn sign_transaction(
        &self,
        tx: &stellar_xdr::curr::Transaction,
    ) -> Result<stellar_xdr::curr::TransactionEnvelope, SorobanHelperError> {
        self.signer.sign_transaction(tx, self.provider.network_id())
    }

    pub async fn send_transaction(
        &self,
        tx_envelope: &stellar_xdr::curr::TransactionEnvelope,
    ) -> Result<stellar_rpc_client::GetTransactionResponse, SorobanHelperError> {
        self.provider.send_transaction(tx_envelope).await
    }

    pub async fn simulate_transaction(
        &self,
        tx_envelope: &stellar_xdr::curr::TransactionEnvelope,
    ) -> Result<stellar_rpc_client::SimulateTransactionResponse, SorobanHelperError> {
        self.provider.simulate_transaction(tx_envelope).await
    }
}
