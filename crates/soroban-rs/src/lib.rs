mod account;
mod contract;
mod crypto;
mod env;
mod error;
mod fs;
mod guard;
pub mod macros;
mod mock;
mod operation;
mod parser;
mod response;
mod rpc;
mod scval;
mod signer;
mod transaction;

pub use account::{Account, AccountConfig, MultisigAccount, SingleAccount};
pub use contract::{ClientContractConfigs, Contract};
pub use env::{Env, EnvConfigs};
pub use error::SorobanHelperError;
pub use guard::{AuthorizedCallsForContract, Guard};
pub use operation::Operations;
pub use parser::{ParseResult, Parser, ParserType};
pub use response::SorobanTransactionResponse;
pub use signer::Signer;
pub use transaction::TransactionBuilder;

// Re-export mock utilities for testing
pub use mock::transaction::{
    create_contract_id_val, create_mock_set_options_tx_envelope,
    mock_transaction_response_with_account_entry, mock_transaction_response_with_return_value,
    MockGetTransactionResponse, MockTransactionMeta, MockTransactionResult,
};

pub use stellar_rpc_client::GetTransactionResponse;
pub use stellar_strkey::Contract as ContractId;

// re-exports
pub use stellar_xdr::curr as xdr;

// traits
pub use scval::IntoScVal;
