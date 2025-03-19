//! # Soroban Transaction Building
//!
//! This module provides functionality for creating and configuring Stellar transactions
//! for use with Soroban smart contracts. It handles transaction construction, fee calculation,
//! and simulation to ensure transactions are properly resourced.
//!
//! ## Features
//!
//! - Transaction building
//! - Automatic sequence number handling
//! - Fee calculation based on transaction simulation
//! - Transaction optimization through simulation
//!
//! ## Example
//!
//! ```rust,no_run
//! use soroban_rs::{Account, Env, EnvConfigs, TransactionBuilder};
//! use stellar_xdr::curr::{Memo, Operation, Preconditions};
//!
//! async fn example(account: &mut Account, env: &Env, operation: Operation) {
//!     // Create a transaction builder
//!     let tx_builder = TransactionBuilder::new(account, env)
//!         .add_operation(operation)
//!         .set_memo(Memo::Text("Example transaction".try_into().unwrap()))
//!         .set_preconditions(Preconditions::None);
//!     
//!     // Build a transaction with simulation to set proper fees
//!     let tx = tx_builder.simulate_and_build(env, account).await?;
//!     
//!     // Sign and submit the transaction
//!     let tx_envelope = account.sign_transaction(&tx, &env.network_id())?;
//!     env.send_transaction(&tx_envelope).await?;
//! }
//! ```
use crate::{Account, Env, error::SorobanHelperError};
use stellar_xdr::curr::{
    Memo, Operation, Preconditions, SequenceNumber, Transaction, TransactionExt,
};

/// Default transaction fee in stroops (0.00001 XLM)
pub const DEFAULT_TRANSACTION_FEES: u32 = 100;

/// Builder for creating and configuring Stellar transactions.
///
/// TransactionBuilder provides an API for building Stellar transactions
/// for Soroban operations. It handles sequence number retrieval, fee calculation,
/// and transaction simulation to ensure transactions have the correct resources
/// allocated for Soroban execution.
#[derive(Clone)]
pub struct TransactionBuilder {
    /// Transaction fee in stroops
    pub fee: u32,
    /// Account that will be the source of the transaction
    pub source_account: Account,
    /// List of operations to include in the transaction
    pub operations: Vec<Operation>,
    /// Optional memo to attach to the transaction
    pub memo: Memo,
    /// Optional preconditions for transaction execution
    pub preconditions: Preconditions,
    /// Environment for network interaction
    pub env: Env,
}

impl TransactionBuilder {
    /// Creates a new transaction builder for the specified account and environment.
    ///
    /// The builder is initialized with default values:
    /// - Default transaction fee
    /// - Empty operations list
    /// - No memo
    /// - No preconditions
    ///
    /// # Parameters
    ///
    /// * `source_account` - The account that will be the source of the transaction
    /// * `env` - The environment for network interaction
    ///
    /// # Returns
    ///
    /// A new TransactionBuilder instance
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

    /// Sets the environment for the transaction builder.
    ///
    /// # Parameters
    ///
    /// * `env` - The new environment to use
    ///
    /// # Returns
    ///
    /// The updated TransactionBuilder
    pub fn set_env(mut self, env: Env) -> Self {
        self.env = env;
        self
    }

    /// Adds an operation to the transaction.
    ///
    /// https://developers.stellar.org/docs/learn/fundamentals/transactions/operations-and-transactions#operations
    ///
    /// # Parameters
    ///
    /// * `operation` - The operation to add
    ///
    /// # Returns
    ///
    /// The updated TransactionBuilder
    pub fn add_operation(mut self, operation: Operation) -> Self {
        self.operations.push(operation);
        self
    }

    /// Sets the memo for the transaction.
    ///
    /// Memos can be used to attach additional information to a transaction.
    /// They are not used by the protocol but can be used by applications.
    /// https://developers.stellar.org/docs/learn/encyclopedia/transactions-specialized/memos
    ///
    /// # Parameters
    ///
    /// * `memo` - The memo to set
    ///
    /// # Returns
    ///
    /// The updated TransactionBuilder
    pub fn set_memo(mut self, memo: Memo) -> Self {
        self.memo = memo;
        self
    }

    /// Sets the preconditions for the transaction.
    ///
    /// Preconditions specify requirements that must be met for a transaction
    /// to be valid, such as time bounds or ledger bounds.
    /// https://developers.stellar.org/docs/learn/fundamentals/transactions/operations-and-transactions#preconditions
    ///
    /// # Parameters
    ///
    /// * `preconditions` - The preconditions to set
    ///
    /// # Returns
    ///
    /// The updated TransactionBuilder
    pub fn set_preconditions(mut self, preconditions: Preconditions) -> Self {
        self.preconditions = preconditions;
        self
    }

    /// Builds a transaction without simulation.
    ///
    /// This method retrieves the source account's current sequence number
    /// and constructs a transaction with the configured parameters.
    ///
    /// # Returns
    ///
    /// A transaction ready to be signed, or an error if the build fails
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Operations cannot be converted to XDR
    /// - Sequence number cannot be retrieved
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

    /// Builds a transaction with simulation to determine proper fees and resources.
    ///
    /// This method:
    /// 1. Builds a transaction with default fees
    /// 2. Simulates the transaction to determine required resources
    /// 3. Updates the transaction with the correct fees and resource data
    ///
    /// This is the recommended way to build Soroban transactions, as it ensures
    /// they have sufficient fees and resources for execution.
    ///
    /// # Parameters
    ///
    /// * `env` - The environment for transaction simulation
    /// * `account` - The account to use for signing the simulation transaction
    ///
    /// # Returns
    ///
    /// A transaction optimized for Soroban execution, or an error if the build fails
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Transaction building fails
    /// - Transaction signing fails
    /// - Simulation fails
    /// - Fee calculation results in a value too large for u32
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

#[cfg(test)]
mod test {
    use crate::{
        Account, TransactionBuilder,
        mock::mocks::{
            mock_account_entry, mock_contract_id, mock_env, mock_signer1, mock_simulate_tx_response,
        },
        operation::Operations,
        transaction::DEFAULT_TRANSACTION_FEES,
    };

    #[tokio::test]
    async fn test_build_transaction() {
        let account = Account::single(mock_signer1());
        let get_account_result = Ok(mock_account_entry(&account.account_id().0.to_string()));

        let env = mock_env(Some(get_account_result), None, None);
        let contract_id = mock_contract_id(account.clone(), &env);
        let operation = Operations::invoke_contract(&contract_id, "test", vec![]).unwrap();
        let transaction = TransactionBuilder::new(&account, &env)
            .add_operation(operation)
            .build()
            .await
            .unwrap();

        assert!(transaction.source_account.account_id() == account.account_id());
        assert!(transaction.operations.len() == 1);
        assert!(transaction.fee == DEFAULT_TRANSACTION_FEES);
    }

    #[tokio::test]
    async fn test_simulate_and_build() {
        let simulation_fee = 42;

        let account = Account::single(mock_signer1());
        let get_account_result = Ok(mock_account_entry(&account.account_id().0.to_string()));
        let simulate_tx_result = Ok(mock_simulate_tx_response(Some(simulation_fee)));

        let env = mock_env(Some(get_account_result), Some(simulate_tx_result), None);
        let contract_id = mock_contract_id(account.clone(), &env);
        let operation = Operations::invoke_contract(&contract_id, "test", vec![]).unwrap();
        let tx_builder = TransactionBuilder::new(&account, &env).add_operation(operation.clone());

        let tx = tx_builder.simulate_and_build(&env, &account).await.unwrap();

        assert!(tx.fee == 142); // DEFAULT_TRANSACTION_FEE + SIMULATION_FEE 
        assert!(tx.operations.len() == 1);
        assert!(tx.operations[0].body == operation.body);
    }
}
