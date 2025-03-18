#[cfg(test)]
pub mod mocks {
    use crate::error::SorobanHelperError;
    use crate::{Signer, rpc::RpcClient};
    use async_trait::async_trait;
    use ed25519_dalek::SigningKey;
    use std::default::Default;
    use std::str::FromStr;
    use stellar_rpc_client::{GetTransactionResponse, SimulateTransactionResponse};
    use stellar_strkey::ed25519::PrivateKey;
    use stellar_xdr::curr::{
        AccountEntry, AccountEntryExt, AccountId, PublicKey, String32, Thresholds,
        TransactionEnvelope, VecM,
    };

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

    pub struct MockRpcClient {}
    impl MockRpcClient {
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
            Ok(SimulateTransactionResponse::default())
        }

        async fn send_transaction_polling(
            &self,
            _tx_envelope: &TransactionEnvelope,
        ) -> Result<GetTransactionResponse, SorobanHelperError> {
            Ok(mock_transaction_response())
        }
    }
}
