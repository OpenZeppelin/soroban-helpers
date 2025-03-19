use crate::error::SorobanHelperError;
use crate::rpc::RpcClient;
use async_trait::async_trait;
use std::default::Default;
use std::sync::RwLock;
use stellar_rpc_client::{GetTransactionResponse, SimulateTransactionResponse};
use stellar_xdr::curr::{ AccountEntry, TransactionEnvelope };

use super::{mock_account_entry, mock_transaction_response};

pub struct MockRpcClient {
    get_account_result: RwLock<Option<Result<AccountEntry, SorobanHelperError>>>,
    simulate_transaction_envelope_result:
        RwLock<Option<Result<SimulateTransactionResponse, SorobanHelperError>>>,
    send_transaction_polling_result:
        RwLock<Option<Result<GetTransactionResponse, SorobanHelperError>>>,
}
impl MockRpcClient {
    pub fn new(
        get_account_result: Option<Result<AccountEntry, SorobanHelperError>>,
        simulate_transaction_envelope_result: Option<
            Result<SimulateTransactionResponse, SorobanHelperError>,
        >,
        send_transaction_polling_result: Option<
            Result<GetTransactionResponse, SorobanHelperError>,
        >,
    ) -> Self {
        Self {
            get_account_result: RwLock::new(get_account_result),
            simulate_transaction_envelope_result: RwLock::new(
                simulate_transaction_envelope_result,
            ),
            send_transaction_polling_result: RwLock::new(send_transaction_polling_result),
        }
    }
}

#[async_trait]
impl RpcClient for MockRpcClient {
    async fn get_account(&self, account_id: &str) -> Result<AccountEntry, SorobanHelperError> {
        let result = self.get_account_result.read().unwrap();
        match result.as_ref() {
            Some(res) => res.clone(),
            None => Ok(mock_account_entry(account_id)),
        }
    }

    async fn simulate_transaction_envelope(
        &self,
        _tx_envelope: &TransactionEnvelope,
    ) -> Result<SimulateTransactionResponse, SorobanHelperError> {
        let result = self.simulate_transaction_envelope_result.read().unwrap();
        match result.as_ref() {
            Some(res) => res.clone(),
            None => Ok(SimulateTransactionResponse::default()),
        }
    }

    async fn send_transaction_polling(
        &self,
        _tx_envelope: &TransactionEnvelope,
    ) -> Result<GetTransactionResponse, SorobanHelperError> {
        let result = self.send_transaction_polling_result.read().unwrap();
        match result.as_ref() {
            Some(res) => res.clone(),
            None => Ok(mock_transaction_response()),
        }
    }
}