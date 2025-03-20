pub mod fs;
pub mod rpc;

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
    AccountEntry, AccountEntryExt, AccountId, Memo, Preconditions, PublicKey, SequenceNumber, String32, Thresholds, Transaction, TransactionEnvelope, TransactionExt, TransactionV1Envelope, Uint256, VecM
};

#[allow(dead_code)]
pub fn mock_transaction(account_id: AccountId) -> Transaction {
    Transaction {
        fee: 100,
        seq_num: SequenceNumber::from(1),
        source_account: account_id.into(),
        cond: Preconditions::None,
        memo: Memo::None,
        operations: VecM::default(),
        ext: TransactionExt::V0,
    }
}

#[allow(dead_code)]
pub fn mock_transaction_envelope(account_id: AccountId) -> TransactionEnvelope {
    TransactionEnvelope::Tx(TransactionV1Envelope {
        tx: mock_transaction(account_id),
        signatures: VecM::default(),
    })
}

#[allow(dead_code)]
pub fn mock_contract_id(account: Account, env: &Env) -> ContractStrKey {
    crypto::calculate_contract_id(&account.account_id(), &Uint256([0; 32]), &env.network_id())
        .unwrap()
}

#[allow(dead_code)]
pub fn mock_simulate_tx_response(min_resource_fee: Option<u64>) -> SimulateTransactionResponse {
    SimulateTransactionResponse {
        min_resource_fee: min_resource_fee.unwrap_or(100),
        transaction_data: "test".to_string(),
        ..Default::default()
    }
}

#[allow(dead_code)]
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
        rpc_client: Arc::new(rpc::MockRpcClient::new(
            get_account_result,
            simulate_transaction_envelope_result,
            send_transaction_polling_result,
        )),
    }
}

#[allow(dead_code)]
pub fn all_signers() -> Vec<Signer> {
    vec![mock_signer1(), mock_signer2(), mock_signer3()]
}

#[allow(dead_code)]
pub fn mock_signer1() -> Signer {
    let pk = PrivateKey::from_string("SD3C2X7WPTUYX4YHL2G34PX75JZ35QJDFKM6SXDLYHWIPOWPIQUXFVLE")
        .unwrap();
    Signer::new(SigningKey::from_bytes(&pk.0))
}

#[allow(dead_code)]
pub fn mock_signer2() -> Signer {
    let pk = PrivateKey::from_string("SDFLNQOG3PV4CYJ4BNUXFXJBBOCQ57MK2NYUK4XUVVJTT2JSA3YDJA3A")
        .unwrap();
    Signer::new(SigningKey::from_bytes(&pk.0))
}

#[allow(dead_code)]
pub fn mock_signer3() -> Signer {
    let pk = PrivateKey::from_string("SASAXDSRHPRZ55OLOD4EWXIWODQEZPYGIBFYX3XBUZGFFVY7QKLYRF5K")
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

#[allow(dead_code)]
pub fn mock_simulate_transaction_response() -> SimulateTransactionResponse {
    SimulateTransactionResponse::default()
}