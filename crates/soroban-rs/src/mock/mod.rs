#[cfg(test)]
pub mod mocks {
    use crate::error::SorobanHelperError;
    use crate::fs::FileReader;
    use crate::{crypto, Account, Env, EnvConfigs};
    use crate::{Signer, rpc::RpcClient};
    use async_trait::async_trait;
    use ed25519_dalek::SigningKey;
    use stellar_strkey::Contract as ContractStrKey;
    use std::cell::RefCell;
    use std::default::Default;
    use std::str::FromStr;
    use std::sync::{Arc, RwLock};
    use stellar_rpc_client::{GetTransactionResponse, SimulateTransactionResponse};
    use stellar_strkey::ed25519::PrivateKey;
    use stellar_xdr::curr::{
        AccountEntry, AccountEntryExt, AccountId, PublicKey, String32, Thresholds, TransactionEnvelope, Uint256, VecM
    };

    pub fn mock_contract_id(account: Account, env: &Env) -> ContractStrKey {
        crypto::calculate_contract_id(
            &account.account_id(),
            &Uint256([0; 32]),
            &env.network_id()
        ).unwrap()
    }

    pub fn mock_simulate_tx_response(min_resource_fee: Option<u64>) -> SimulateTransactionResponse {
        let mut response = SimulateTransactionResponse::default();
        response.min_resource_fee = min_resource_fee.unwrap_or(100);
        response.transaction_data = "test".to_string();
        response
    }

    pub fn mock_env(
        get_account_result: Option<Result<AccountEntry, SorobanHelperError>>,
        simulate_transaction_envelope_result: Option<
            Result<SimulateTransactionResponse, SorobanHelperError>,
        >,
        send_transaction_polling_result: Option<Result<GetTransactionResponse, SorobanHelperError>>,
    ) -> Env {
        Env {
            configs: EnvConfigs {
                rpc_url: "http://test.com".to_string(),
                network_passphrase: "test".to_string(),
            },
            rpc_client: Arc::new(MockRpcClient::new(
                get_account_result,
                simulate_transaction_envelope_result,
                send_transaction_polling_result,
            )),
        }
    }

    pub fn all_signers() -> Vec<Signer> {
        vec![mock_signer1(), mock_signer2(), mock_signer3()]
    }

    pub fn mock_signer1() -> Signer {
        let pk =
            PrivateKey::from_string("SD3C2X7WPTUYX4YHL2G34PX75JZ35QJDFKM6SXDLYHWIPOWPIQUXFVLE")
                .unwrap();
        Signer::new(SigningKey::from_bytes(&pk.0))
    }

    pub fn mock_signer2() -> Signer {
        let pk =
            PrivateKey::from_string("SDFLNQOG3PV4CYJ4BNUXFXJBBOCQ57MK2NYUK4XUVVJTT2JSA3YDJA3A")
                .unwrap();
        Signer::new(SigningKey::from_bytes(&pk.0))
    }

    pub fn mock_signer3() -> Signer {
        let pk =
            PrivateKey::from_string("SASAXDSRHPRZ55OLOD4EWXIWODQEZPYGIBFYX3XBUZGFFVY7QKLYRF5K")
                .unwrap();
        Signer::new(SigningKey::from_bytes(&pk.0))
    }

    pub fn mock_account_entry(account_id: &str) -> AccountEntry {
        AccountEntry {
            account_id: AccountId(PublicKey::from_str(account_id).unwrap()),
            balance: 0,
            ext: AccountEntryExt::V0,
            flags: 0,
            home_domain: String32::default(),
            inflation_dest: None,
            seq_num: 0.into(),
            num_sub_entries: 0,
            signers: VecM::default(),
            thresholds: Thresholds([0, 0, 0, 0]),
        }
    }

    pub fn mock_transaction_response() -> GetTransactionResponse {
        GetTransactionResponse {
            envelope: None,
            result: None,
            result_meta: None,
            status: "".to_string(),
        }
    }

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

    pub struct MockFileReader {
        // Use RefCell to allow modifying the value inside the mock
        mock_data: RefCell<Result<Vec<u8>, SorobanHelperError>>,
    }

    impl MockFileReader {
        pub fn new(mock_data: Result<Vec<u8>, SorobanHelperError>) -> Self {
            MockFileReader {
                mock_data: RefCell::new(mock_data),
            }
        }
    }

    impl FileReader for MockFileReader {
        fn read(&self, _path: &str) -> Result<Vec<u8>, SorobanHelperError> {
            self.mock_data.borrow().clone()
        }
    }
}
