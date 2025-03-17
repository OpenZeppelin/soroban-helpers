pub mod mocks {
    use std::str::FromStr;
    use crate::error::SorobanHelperError;
    use crate::{Signer, rpc::RpcClient};
    use async_trait::async_trait;
    use ed25519_dalek::SigningKey;
    use stellar_rpc_client::{GetTransactionResponse, SimulateTransactionResponse};
    use stellar_strkey::ed25519::PrivateKey;
    use stellar_xdr::curr::{
        AccountEntry, AccountEntryExt, AccountId, PublicKey, String32, Thresholds,
        TransactionEnvelope, VecM,
    };

    pub fn mock_signer() -> Signer {
        let pk =
            PrivateKey::from_string("SD3C2X7WPTUYX4YHL2G34PX75JZ35QJDFKM6SXDLYHWIPOWPIQUXFVLE")
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

    pub struct MockRpcClient {}
    impl MockRpcClient {
        #[allow(dead_code)]
        pub fn new() -> Self {
            Self {}
        }
    }

    #[async_trait]
    impl RpcClient for MockRpcClient {
        async fn get_account(&self, account_id: &str) -> Result<AccountEntry, SorobanHelperError> {
            Ok(mock_account_entry(account_id))
        }

        async fn simulate_transaction_envelope(
            &self,
            _tx_envelope: &TransactionEnvelope,
        ) -> Result<SimulateTransactionResponse, SorobanHelperError> {
            Err(SorobanHelperError::InvalidArgument(
                "not implemented".to_string(),
            ))
        }

        async fn send_transaction_polling(
            &self,
            _tx_envelope: &TransactionEnvelope,
        ) -> Result<GetTransactionResponse, SorobanHelperError> {
            Err(SorobanHelperError::InvalidArgument(
                "not implemented".to_string(),
            ))
        }
    }
}
