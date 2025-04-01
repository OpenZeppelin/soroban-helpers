//! # Soroban Account Guards
//! Represents a guard mechanism that can
//! be used to control and limit operations.
//!
//! ## Example
//!
//! ```rust,no_run
//! use soroban_rs::{Env, Signer, Account, Guard};
//! use ed25519_dalek::SigningKey;
//!
//! async fn example(signing_key: SigningKey) {
//!     let mut account = Account::single(Signer::new(signing_key));
//!     let guard = Guard::NumberOfAllowedCalls(3);
//!     account.add_guard(guard);
//! }
//! ```
use stellar_strkey::Contract as ContractId;
use stellar_xdr::curr::{HostFunction, OperationBody, Transaction};

use crate::SorobanHelperError;

#[derive(Clone)]
pub enum Guard {
    /// Limits the number of allowed calls to a specific operation.
    /// The u16 value represents the remaining number of calls allowed.
    NumberOfAllowedCalls(u16),
    AuthorizedCallsFor(AuthorizedCallsForContract),
    // ... other variants
}

impl Guard {
    /// Checks if the guard condition is satisfied.
    ///
    /// # Returns
    /// * `true` if the operation is allowed to proceed
    /// * `false` if the operation should be blocked
    pub fn check(&self, transaction: &Transaction) -> Result<bool, SorobanHelperError> {
        match self {
            Guard::NumberOfAllowedCalls(remaining) => Ok(*remaining > 0),
            Guard::AuthorizedCallsFor(calls_for_contract) => {
                Ok(calls_for_contract.check(transaction))
            }
            // handle other variants
        }
    }

    /// Updates the guard state after an operation has been performed.
    ///
    /// This method should be called after a successful operation to update
    /// the internal state of the guard (e.g., decrement remaining allowed calls).
    pub fn update(&mut self, transaction: &Transaction) -> Result<(), SorobanHelperError> {
        match self {
            Guard::NumberOfAllowedCalls(remaining) => {
                if *remaining > 0 {
                    *remaining -= 1;
                }
                Ok(())
            }
            Guard::AuthorizedCallsFor(calls_for_contract) => {
                calls_for_contract.update(transaction);
                Ok(())
            }
            // handle other variants
        }
    }
}

#[derive(Clone)]
pub struct AuthorizedCallsForContract {
    pub contract_id: ContractId,
    pub remaining: u16,
}

impl AuthorizedCallsForContract {
    fn extract_contract_calls(&self, tx: &Transaction) -> u16 {
        let mut calls = 0;
        for op in tx.operations.iter() {
            if let OperationBody::InvokeHostFunction(invoke_op) = &op.body {
                if let HostFunction::InvokeContract(args) = &invoke_op.host_function {
                    let addr_string = args.contract_address.to_string();
                    if addr_string == self.contract_id.to_string() {
                        calls += 1;
                    }
                }
            }
        }
        calls
    }

    pub fn check(&self, transaction: &Transaction) -> bool {
        let calls = self.extract_contract_calls(transaction);
        self.remaining >= calls && calls > 0
    }

    pub fn update(&mut self, transaction: &Transaction) {
        let calls = self.extract_contract_calls(transaction);
        if calls > 0 && self.remaining >= calls {
            self.remaining -= calls;
        }
    }
}
