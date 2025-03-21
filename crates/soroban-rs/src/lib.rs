mod account;
mod contract;
mod crypto;
mod env;
mod error;
mod fs;
mod mock;
mod operation;
mod parser;
mod rpc;
mod signer;
mod transaction;

pub use account::{Account, AccountConfig, MultisigAccount, SingleAccount};
pub use contract::{ClientContractConfigs, Contract};
pub use env::{Env, EnvConfigs};
pub use parser::{ParseResult, Parser, ParserType};
pub use signer::Signer;
pub use transaction::TransactionBuilder;

// Re-export mock parser utilities for testing
pub use mock::parser::{
    MockGetTransactionResponse, MockTransactionMeta, MockTransactionResult,
    create_mock_set_options_tx_envelope, mock_to_real_response,
};

// Re-export mock functions from transaction module
pub use mock::transaction::{
    create_contract_id_val, mock_transaction_response_with_account_entry,
    mock_transaction_response_with_return_value,
};

pub use stellar_xdr::curr as xdr;
