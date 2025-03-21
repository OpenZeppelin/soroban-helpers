use stellar_rpc_client::{GetTransactionResponse, SimulateTransactionResponse};
use stellar_xdr::curr::{
    AccountEntry, AccountId, Memo, Preconditions, ScVal, SequenceNumber,
    Transaction, TransactionExt, VecM,
};

/// Creates a basic transaction for mocking purposes
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

/// Creates a mock SimulateTransactionResponse with a specified resource fee
#[allow(dead_code)]
pub fn mock_simulate_tx_response(min_resource_fee: Option<u64>) -> SimulateTransactionResponse {
    SimulateTransactionResponse {
        min_resource_fee: min_resource_fee.unwrap_or(100),
        transaction_data: "test".to_string(),
        ..Default::default()
    }
}

/// Creates a basic empty transaction response
#[allow(dead_code)]
pub fn mock_transaction_response() -> GetTransactionResponse {
    GetTransactionResponse {
        envelope: None,
        result: None,
        result_meta: None,
        status: "".to_string(),
    }
}

/// Creates a successful transaction response with return value
#[allow(dead_code)]
pub fn mock_transaction_response_with_return_value(return_val: ScVal) -> GetTransactionResponse {
    use stellar_xdr::curr::{
        SorobanTransactionMeta, SorobanTransactionMetaExt, TransactionMeta,
        TransactionMetaV3, TransactionResult, TransactionResultExt, TransactionResultResult,
        ExtensionPoint, VecM,
    };
    
    // Create success result
    let result = Some(TransactionResult {
        fee_charged: 100,
        result: TransactionResultResult::TxSuccess(VecM::default()),
        ext: TransactionResultExt::V0,
    });
    
    // Create metadata with return value
    let meta = Some(TransactionMeta::V3(TransactionMetaV3 {
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
    }));
    
    GetTransactionResponse {
        status: "SUCCESS".to_string(),
        envelope: None,
        result,
        result_meta: meta,
    }
}

/// Creates a successful transaction response with account entry
#[allow(dead_code)]
pub fn mock_transaction_response_with_account_entry(account: AccountEntry) -> GetTransactionResponse {
    use stellar_xdr::curr::{
        ExtensionPoint, LedgerEntry, LedgerEntryChange, LedgerEntryData, LedgerEntryExt,
        OperationMeta, TransactionMeta, TransactionMetaV3, TransactionResult, TransactionResultExt,
        TransactionResultResult, VecM,
    };
    
    // Create success result
    let result = Some(TransactionResult {
        fee_charged: 100,
        result: TransactionResultResult::TxSuccess(VecM::default()),
        ext: TransactionResultExt::V0,
    });
    
    // Create a ledger entry for the account
    let ledger_entry = LedgerEntry {
        last_modified_ledger_seq: 1,
        data: LedgerEntryData::Account(account),
        ext: LedgerEntryExt::V0,
    };
    
    // Create a change for the updated account
    let change = LedgerEntryChange::Updated(ledger_entry);
    
    // Create a VecM of changes
    let changes = VecM::try_from(vec![change]).unwrap_or_default();
    
    // Create an operation meta with the changes
    let op_meta = OperationMeta {
        changes: stellar_xdr::curr::LedgerEntryChanges(changes),
    };
    
    // Create a VecM of operation metas
    let operations = VecM::try_from(vec![op_meta]).unwrap_or_default();
    
    // Create metadata with account changes
    let meta = Some(TransactionMeta::V3(TransactionMetaV3 {
        ext: ExtensionPoint::V0,
        soroban_meta: None,
        tx_changes_before: Default::default(),
        tx_changes_after: Default::default(),
        operations,
    }));
    
    GetTransactionResponse {
        status: "SUCCESS".to_string(),
        envelope: None,
        result,
        result_meta: meta,
    }
}

/// Creates a contract ID ScVal for testing
#[allow(dead_code)]
pub fn create_contract_id_val() -> ScVal {
    let contract_hash = stellar_xdr::curr::Hash([1; 32]);
    let address = stellar_xdr::curr::ScAddress::Contract(contract_hash);
    stellar_xdr::curr::ScVal::Address(address)
} 