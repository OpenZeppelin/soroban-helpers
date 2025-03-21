use stellar_rpc_client::{GetTransactionResponse, SimulateTransactionResponse};
use stellar_xdr::curr::{
    AccountEntry, AccountId, ExtensionPoint, Memo, OperationResult, Preconditions, ScVal, 
    SequenceNumber, SorobanTransactionMeta, SorobanTransactionMetaExt, Transaction,
    TransactionExt, TransactionMeta, TransactionMetaV3, TransactionResult,
    TransactionResultExt, TransactionResultResult, VecM,
};

/// Simple mock structure for transaction results
pub struct MockTransactionResult {
    pub success: bool,
}

/// Simple mock structure for transaction metadata
pub struct MockTransactionMeta {
    pub return_value: Option<ScVal>,
    pub account_entry: Option<AccountEntry>,
}

/// Mock transaction response for testing
pub struct MockGetTransactionResponse {
    /// The transaction result, if available
    pub tx_result: Option<MockTransactionResult>,
    /// The transaction metadata, if available
    pub tx_meta: Option<MockTransactionMeta>,
    /// The transaction envelope, if available
    pub tx_envelope: Option<stellar_xdr::curr::TransactionEnvelope>,
}

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

/// Creates a basic transaction response that indicates success
#[allow(dead_code)]
pub fn mock_transaction_response() -> GetTransactionResponse {
    use stellar_xdr::curr::{
        TransactionResult, TransactionResultExt, TransactionResultResult, VecM,
    };

    // Create success result
    let result = Some(TransactionResult {
        fee_charged: 100,
        result: TransactionResultResult::TxSuccess(VecM::default()),
        ext: TransactionResultExt::V0,
    });

    GetTransactionResponse {
        envelope: None,
        result,
        result_meta: None,
        status: "SUCCESS".to_string(),
    }
}

/// Creates a successful transaction response with return value
#[allow(dead_code)]
pub fn mock_transaction_response_with_return_value(return_val: ScVal) -> GetTransactionResponse {
    use stellar_xdr::curr::{
        ExtensionPoint, SorobanTransactionMeta, SorobanTransactionMetaExt, TransactionMeta,
        TransactionMetaV3, TransactionResult, TransactionResultExt, TransactionResultResult, VecM,
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

/// Creates a basic transaction envelope for testing
#[allow(dead_code)]
pub fn mock_transaction_envelope(account_id: stellar_xdr::curr::AccountId) -> stellar_xdr::curr::TransactionEnvelope {
    use stellar_xdr::curr::{
        Memo, Preconditions, Transaction, TransactionEnvelope, TransactionExt,
        TransactionV1Envelope,
    };

    TransactionEnvelope::Tx(TransactionV1Envelope {
        tx: Transaction {
            source_account: account_id.into(),
            fee: 100,
            seq_num: 1.into(),
            cond: Preconditions::None,
            memo: Memo::None,
            operations: VecM::default(),
            ext: TransactionExt::V0,
        },
        signatures: Default::default(),
    })
}

/// Create a success transaction result
#[allow(dead_code)]
pub fn create_success_tx_result() -> TransactionResult {
    // Create empty operation results
    let empty_vec: Vec<OperationResult> = Vec::new();
    let op_results = empty_vec.try_into().unwrap_or_default();

    TransactionResult {
        fee_charged: 100,
        result: TransactionResultResult::TxSuccess(op_results),
        ext: TransactionResultExt::V0,
    }
}

/// Create a transaction meta from mock
#[allow(dead_code)]
pub fn create_tx_meta_from_mock(mock: &MockTransactionMeta) -> TransactionMeta {
    // Check if we have a return value
    if let Some(return_val) = &mock.return_value {
        return create_soroban_tx_meta_with_return_value(return_val.clone());
    }

    // Check if we have an account entry
    if let Some(account) = &mock.account_entry {
        // Create a V3 meta with account entry in the operations changes
        use stellar_xdr::curr::{
            LedgerEntry, LedgerEntryChange, LedgerEntryData, LedgerEntryExt, OperationMeta, VecM,
        };

        // Create a ledger entry for the account
        let ledger_entry = LedgerEntry {
            last_modified_ledger_seq: 1,
            data: LedgerEntryData::Account(account.clone()),
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

        // Return the transaction meta with operations
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

/// Create a transaction meta with return value
#[allow(dead_code)]
pub fn create_soroban_tx_meta_with_return_value(return_val: ScVal) -> TransactionMeta {
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

/// Create a mock transaction envelope with SetOptions operation
#[allow(dead_code)]
pub fn create_mock_set_options_tx_envelope() -> stellar_xdr::curr::TransactionEnvelope {
    use stellar_xdr::curr::{
        Memo, MuxedAccount, Operation, OperationBody, Preconditions, SetOptionsOp, Transaction,
        TransactionEnvelope, TransactionExt, TransactionV1Envelope, Uint256,
    };

    TransactionEnvelope::Tx(TransactionV1Envelope {
        tx: Transaction {
            source_account: MuxedAccount::Ed25519(Uint256([0; 32])),
            fee: 100,
            seq_num: 1.into(),
            cond: Preconditions::None,
            memo: Memo::None,
            operations: vec![Operation {
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
            }]
            .try_into()
            .unwrap(),
            ext: TransactionExt::V0,
        },
        signatures: Default::default(),
    })
}

/// Helper to convert a mock to a real transaction response
#[allow(dead_code)]
pub fn mock_to_real_response(mock: &MockGetTransactionResponse) -> GetTransactionResponse {
    // Create a real GetTransactionResponse with the data from our mock
    GetTransactionResponse {
        status: "SUCCESS".to_string(),
        envelope: mock.tx_envelope.clone(),
        result: mock.tx_result.as_ref().map(|_| create_success_tx_result()),
        result_meta: mock
            .tx_meta
            .as_ref()
            .map(create_tx_meta_from_mock),
    }
}
