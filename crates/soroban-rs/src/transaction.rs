use crate::{Account, Env, error::SorobanHelperError};
use stellar_xdr::curr::{
    Memo, Operation, Preconditions, SequenceNumber, Transaction, TransactionExt,
};

pub const DEFAULT_TRANSACTION_FEES: u32 = 100;

#[derive(Clone)]
pub struct TransactionBuilder {
    pub fee: u32,
    pub source_account: Account,
    pub operations: Vec<Operation>,
    pub memo: Memo,
    pub preconditions: Preconditions,
    pub env: Env,
}

impl TransactionBuilder {
    pub fn new(source_account: &Account, env: &Env) -> Self {
        Self {
            fee: DEFAULT_TRANSACTION_FEES,
            source_account: source_account.clone(),
            operations: Vec::new(),
            memo: Memo::None,
            preconditions: Preconditions::None,
            env: env.clone(),
        }
    }

    pub fn set_env(mut self, env: Env) -> Self {
        self.env = env;
        self
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

    pub async fn build(self) -> Result<Transaction, SorobanHelperError> {
        let operations = self.operations.try_into().map_err(|e| {
            SorobanHelperError::XdrEncodingFailed(format!("Failed to convert operations: {}", e))
        })?;

        let seq_num = self
            .source_account
            .get_sequence(&self.env)
            .await
            .map_err(|e| {
                SorobanHelperError::XdrEncodingFailed(format!(
                    "Failed to get sequence number: {}",
                    e
                ))
            })?;

        Ok(Transaction {
            fee: self.fee,
            seq_num: SequenceNumber::from(seq_num.increment().value()),
            source_account: self.source_account.account_id().into(),
            cond: self.preconditions,
            memo: self.memo,
            operations,
            ext: TransactionExt::V0,
        })
    }

    pub async fn simulate_and_build(
        self,
        env: &Env,
        account: &Account,
    ) -> Result<Transaction, SorobanHelperError> {
        let tx = self.build().await?;
        let tx_envelope = account.sign_transaction_unsafe(&tx, &env.network_id())?;
        let simulation = env.simulate_transaction(&tx_envelope).await?;

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
