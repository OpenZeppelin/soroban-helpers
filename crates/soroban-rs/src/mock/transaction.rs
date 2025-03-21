use std::convert::TryInto;
use stellar_rpc_client::{GetTransactionResponse, SimulateTransactionResponse};
use stellar_xdr::curr::{
    AccountEntry, AccountId, ExtensionPoint, Hash, LedgerEntry, LedgerEntryChange, LedgerEntryData,
    LedgerEntryExt, Memo, MuxedAccount, Operation, OperationBody, OperationMeta, OperationResult,
    Preconditions, ScAddress, ScVal, SequenceNumber, SetOptionsOp, SorobanTransactionMeta,
    SorobanTransactionMetaExt, Transaction, TransactionEnvelope, TransactionExt, TransactionMeta,
    TransactionMetaV3, TransactionResult, TransactionResultExt, TransactionResultResult,
    TransactionV1Envelope, Uint256, VecM,
};

pub struct MockTransactionResult {
    pub success: bool,
}

pub struct MockTransactionMeta {
    pub return_value: Option<ScVal>,
    pub account_entry: Option<AccountEntry>,
}

pub struct MockGetTransactionResponse {
    pub tx_result: Option<MockTransactionResult>,
    pub tx_meta: Option<MockTransactionMeta>,
    pub tx_envelope: Option<TransactionEnvelope>,
}

enum MockResponseType {
    Basic,
    WithReturnValue(ScVal),
    WithAccountEntry(AccountEntry),
}

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
        signatures: Default::default(),
    })
}

#[allow(dead_code)]
pub fn create_contract_id_val() -> ScVal {
    let contract_hash = Hash([1; 32]);
    ScVal::Address(ScAddress::Contract(contract_hash))
}

#[allow(dead_code)]
pub fn create_mock_set_options_tx_envelope() -> TransactionEnvelope {
    // Create a mock account source
    let source_account = MuxedAccount::Ed25519(Uint256([0; 32]));

    // Create a SetOptions operation
    let set_options_op = Operation {
        source_account: None,
        body: OperationBody::SetOptions(SetOptionsOp {
            inflation_dest: None,
            clear_flags: None,
            set_flags: None,
            master_weight: None,
            low_threshold: None,
            med_threshold: None,
            high_threshold: None,
            home_domain: None,
            signer: None,
        }),
    };

    // Create the operations vector
    let operations = vec![set_options_op].try_into().unwrap_or_default();

    // Create and return the transaction envelope
    TransactionEnvelope::Tx(TransactionV1Envelope {
        tx: Transaction {
            source_account,
            fee: 100,
            seq_num: 1.into(),
            cond: Preconditions::None,
            memo: Memo::None,
            operations,
            ext: TransactionExt::V0,
        },
        signatures: Default::default(),
    })
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
fn mock_transaction_response_impl(response_type: MockResponseType) -> GetTransactionResponse {
    let mut response = GetTransactionResponse {
        status: "SUCCESS".to_string(),
        envelope: None,
        result: Some(create_success_tx_result()),
        result_meta: None,
    };

    match response_type {
        MockResponseType::Basic => {}
        MockResponseType::WithReturnValue(val) => {
            response.result_meta = Some(create_soroban_tx_meta_with_return_value(val));
        }
        MockResponseType::WithAccountEntry(account) => {
            let ledger_entry = LedgerEntry {
                last_modified_ledger_seq: 1,
                data: LedgerEntryData::Account(account),
                ext: LedgerEntryExt::V0,
            };

            let change = LedgerEntryChange::Updated(ledger_entry);
            let changes = VecM::try_from(vec![change]).unwrap_or_default();
            let op_meta = OperationMeta {
                changes: stellar_xdr::curr::LedgerEntryChanges(changes),
            };

            let operations = VecM::try_from(vec![op_meta]).unwrap_or_default();
            let meta = TransactionMeta::V3(TransactionMetaV3 {
                ext: ExtensionPoint::V0,
                soroban_meta: None,
                tx_changes_before: Default::default(),
                tx_changes_after: Default::default(),
                operations,
            });

            response.result_meta = Some(meta);
        }
    }

    response
}

#[allow(dead_code)]
pub fn mock_transaction_response() -> GetTransactionResponse {
    mock_transaction_response_impl(MockResponseType::Basic)
}

#[allow(dead_code)]
pub fn mock_transaction_response_with_return_value(return_val: ScVal) -> GetTransactionResponse {
    mock_transaction_response_impl(MockResponseType::WithReturnValue(return_val))
}

#[allow(dead_code)]
pub fn mock_transaction_response_with_account_entry(
    account: AccountEntry,
) -> GetTransactionResponse {
    mock_transaction_response_impl(MockResponseType::WithAccountEntry(account))
}

#[allow(dead_code)]
pub fn mock_to_real_response(mock: &MockGetTransactionResponse) -> GetTransactionResponse {
    GetTransactionResponse {
        status: "SUCCESS".to_string(),
        envelope: mock.tx_envelope.clone(),
        result: mock.tx_result.as_ref().map(|_| create_success_tx_result()),
        result_meta: mock.tx_meta.as_ref().map(create_tx_meta_from_mock),
    }
}

#[allow(dead_code)]
fn create_success_tx_result() -> TransactionResult {
    // Create empty operation results
    let empty_vec: Vec<OperationResult> = Vec::new();
    let op_results = empty_vec.try_into().unwrap_or_default();

    TransactionResult {
        fee_charged: 100,
        result: TransactionResultResult::TxSuccess(op_results),
        ext: TransactionResultExt::V0,
    }
}

#[allow(dead_code)]
fn create_soroban_tx_meta_with_return_value(return_val: ScVal) -> TransactionMeta {
    TransactionMeta::V3(TransactionMetaV3 {
        ext: ExtensionPoint::V0,
        soroban_meta: Some(SorobanTransactionMeta {
            ext: SorobanTransactionMetaExt::V0,
            events: Default::default(),
            return_value: return_val,
            diagnostic_events: Default::default(),
        }),
        tx_changes_before: Default::default(),
        tx_changes_after: Default::default(),
        operations: Default::default(),
    })
}

#[allow(dead_code)]
fn create_tx_meta_from_mock(mock: &MockTransactionMeta) -> TransactionMeta {
    // Check if we have a return value
    if let Some(return_val) = &mock.return_value {
        return create_soroban_tx_meta_with_return_value(return_val.clone());
    }

    // Check if we have an account entry
    if let Some(account) = &mock.account_entry {
        // Create a ledger entry for the account
        let ledger_entry = LedgerEntry {
            last_modified_ledger_seq: 1,
            data: LedgerEntryData::Account(account.clone()),
            ext: LedgerEntryExt::V0,
        };

        let change = LedgerEntryChange::Updated(ledger_entry);
        let changes = VecM::try_from(vec![change]).unwrap_or_default();
        let op_meta = OperationMeta {
            changes: stellar_xdr::curr::LedgerEntryChanges(changes),
        };

        let operations = VecM::try_from(vec![op_meta]).unwrap_or_default();

        return TransactionMeta::V3(TransactionMetaV3 {
            ext: ExtensionPoint::V0,
            soroban_meta: None,
            tx_changes_before: Default::default(),
            tx_changes_after: Default::default(),
            operations,
        });
    }

    // Default empty meta if neither return value nor account entry is present
    TransactionMeta::V3(TransactionMetaV3 {
        ext: ExtensionPoint::V0,
        soroban_meta: None,
        tx_changes_before: Default::default(),
        tx_changes_after: Default::default(),
        operations: Default::default(),
    })
}
