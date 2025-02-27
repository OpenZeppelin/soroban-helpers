use crate::{Provider, Signer};
use stellar_xdr::curr::{
    Memo, Operation, Preconditions, SequenceNumber, Transaction, TransactionExt,
};

pub const DEFAULT_TRANSACTION_FEES: u32 = 100;

pub struct TransactionBuilder {
    pub fee: u32,
    pub source_account: stellar_xdr::curr::AccountId,
    pub sequence: i64,
    pub operations: Vec<Operation>,
    pub memo: Memo,
    pub preconditions: Preconditions,
}

impl TransactionBuilder {
    pub fn new(source_account: stellar_xdr::curr::AccountId, sequence: i64) -> Self {
        Self {
            fee: DEFAULT_TRANSACTION_FEES,
            source_account,
            sequence,
            operations: Vec::new(),
            memo: Memo::None,
            preconditions: Preconditions::None,
        }
    }

    pub fn add_operation(&mut self, operation: Operation) -> &mut Self {
        self.operations.push(operation);
        self
    }

    pub fn set_memo(&mut self, memo: Memo) -> &mut Self {
        self.memo = memo;
        self
    }

    pub fn set_preconditions(&mut self, preconditions: Preconditions) -> &mut Self {
        self.preconditions = preconditions;
        self
    }

    pub fn build(self) -> Result<Transaction, Box<dyn std::error::Error>> {
        let operations = self.operations.try_into()?;

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
        signer: &Signer,
    ) -> Result<Transaction, Box<dyn std::error::Error>> {
        let tx = self.build()?;
        let tx_envelope = signer.sign_transaction(&tx, provider.network_id())?;
        let simulation = provider.simulate_transaction(&tx_envelope).await?;

        let updated_fee = DEFAULT_TRANSACTION_FEES.max(
            u32::try_from(DEFAULT_TRANSACTION_FEES as u64 + simulation.min_resource_fee)
                .expect("Transaction fee too high"),
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

        if let Ok(tx_data) = simulation.transaction_data() {
            tx.ext = TransactionExt::V1(tx_data);
        }

        Ok(tx)
    }
}
