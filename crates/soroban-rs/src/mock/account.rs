use crate::Signer;
use crate::error::SorobanHelperError;
use crate::{Account, Env, EnvConfigs, crypto};
use ed25519_dalek::SigningKey;
use std::default::Default;
use std::str::FromStr;
use std::sync::Arc;
use stellar_rpc_client::{GetTransactionResponse, SimulateTransactionResponse};
use stellar_strkey::Contract as ContractStrKey;
use stellar_strkey::ed25519::PrivateKey;
use stellar_xdr::curr::{
    AccountEntry, AccountEntryExt, AccountId, PublicKey, String32, Thresholds, Uint256, VecM,
};

use super::rpc::MockRpcClient;

/// Creates a mock contract ID for testing
#[allow(dead_code)]
pub fn mock_contract_id(account: Account, env: &Env) -> ContractStrKey {
    crypto::calculate_contract_id(&account.account_id(), &Uint256([0; 32]), &env.network_id())
        .unwrap()
}

/// Creates a mock environment with configurable responses
#[allow(dead_code)]
pub fn mock_env(
    get_account_result: Option<Result<AccountEntry, SorobanHelperError>>,
    simulate_transaction_envelope_result: Option<
        Result<SimulateTransactionResponse, SorobanHelperError>,
    >,
    send_transaction_polling_result: Option<Result<GetTransactionResponse, SorobanHelperError>>,
) -> Env {
    let random_id = rand::random::<u64>();
    let network_passphrase = format!("Mock Test Random Network {}", random_id);

    Env {
        configs: EnvConfigs {
            rpc_url: "http://test.com".to_string(),
            network_passphrase,
        },
        rpc_client: Arc::new(MockRpcClient::new(
            get_account_result,
            simulate_transaction_envelope_result,
            send_transaction_polling_result,
        )),
    }
}

/// Returns a collection of mock signers for testing
#[allow(dead_code)]
pub fn all_signers() -> Vec<Signer> {
    vec![mock_signer1(), mock_signer2(), mock_signer3()]
}

/// Creates the first mock signer with a predefined private key
#[allow(dead_code)]
pub fn mock_signer1() -> Signer {
    let pk = PrivateKey::from_string("SD3C2X7WPTUYX4YHL2G34PX75JZ35QJDFKM6SXDLYHWIPOWPIQUXFVLE")
        .unwrap();
    Signer::new(SigningKey::from_bytes(&pk.0))
}

/// Creates the second mock signer with a predefined private key
#[allow(dead_code)]
pub fn mock_signer2() -> Signer {
    let pk = PrivateKey::from_string("SDFLNQOG3PV4CYJ4BNUXFXJBBOCQ57MK2NYUK4XUVVJTT2JSA3YDJA3A")
        .unwrap();
    Signer::new(SigningKey::from_bytes(&pk.0))
}

/// Creates the third mock signer with a predefined private key
#[allow(dead_code)]
pub fn mock_signer3() -> Signer {
    let pk = PrivateKey::from_string("SASAXDSRHPRZ55OLOD4EWXIWODQEZPYGIBFYX3XBUZGFFVY7QKLYRF5K")
        .unwrap();
    Signer::new(SigningKey::from_bytes(&pk.0))
}

/// Creates a mock account entry with specified account ID
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
