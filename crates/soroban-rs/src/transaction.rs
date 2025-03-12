use crate::{Account, Provider, error::SorobanHelperError};
use stellar_xdr::curr::{
    AccountId, Memo, Operation, Preconditions, SequenceNumber, Transaction, TransactionExt,
};

pub const DEFAULT_TRANSACTION_FEES: u32 = 100;

pub struct TransactionBuilder {
    pub fee: u32,
    pub source_account: AccountId,
    pub sequence: i64,
    pub operations: Vec<Operation>,
    pub memo: Memo,
    pub preconditions: Preconditions,
}

impl Clone for TransactionBuilder {
    fn clone(&self) -> Self {
        Self {
            fee: self.fee,
            source_account: self.source_account.clone(),
            sequence: self.sequence,
            operations: self.operations.clone(),
            memo: self.memo.clone(),
            preconditions: self.preconditions.clone(),
        }
    }
}

impl TransactionBuilder {
    pub fn new(source_account: AccountId, sequence: i64) -> Self {
        Self {
            fee: DEFAULT_TRANSACTION_FEES,
            source_account,
            sequence,
            operations: Vec::new(),
            memo: Memo::None,
            preconditions: Preconditions::None,
        }
    }

    pub fn add_operation(mut self, operation: Operation) -> Self {
        self.operations.push(operation);
        self
    }

    pub fn set_memo(mut self, memo: Memo) -> Self {
        self.memo = memo;
        self
    }

    pub fn set_preconditions(mut self, preconditions: Preconditions) -> Self {
        self.preconditions = preconditions;
        self
    }

    pub fn build(self) -> Result<Transaction, SorobanHelperError> {
        let operations = self.operations.try_into().map_err(|e| {
            SorobanHelperError::XdrEncodingFailed(format!("Failed to convert operations: {}", e))
        })?;

        Ok(Transaction {
            fee: self.fee,
            seq_num: SequenceNumber(self.sequence),
            source_account: self.source_account.into(),
            cond: self.preconditions,
            memo: self.memo,
            operations,
            ext: TransactionExt::V0,
        })
    }

    pub async fn simulate_and_build(
        self,
        provider: &Provider,
        account: &Account,
    ) -> Result<Transaction, SorobanHelperError> {
        let tx = self.build()?;
        let tx_envelope = account.sign_transaction(&tx, provider.network_id())?;
        let simulation = provider.simulate_transaction(&tx_envelope).await?;

        let updated_fee = DEFAULT_TRANSACTION_FEES.max(
            u32::try_from(
                (tx.operations.len() as u64 * DEFAULT_TRANSACTION_FEES as u64)
                    + simulation.min_resource_fee,
            )
            .map_err(|_| {
                SorobanHelperError::InvalidArgument("Transaction fee too high".to_string())
            })?,
        );

        let mut tx = Transaction {
            fee: updated_fee,
            seq_num: tx.seq_num,
            source_account: tx.source_account,
            cond: tx.cond,
            memo: tx.memo,
            operations: tx.operations,
            ext: tx.ext,
        };

        if let Ok(tx_data) = simulation.transaction_data().map_err(|e| {
            SorobanHelperError::TransactionFailed(format!("Failed to get transaction data: {}", e))
        }) {
            tx.ext = TransactionExt::V1(tx_data);
        }

        Ok(tx)
    }
}
