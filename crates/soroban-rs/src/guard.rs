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
use stellar_xdr::curr::{
    OperationBody, SorobanAuthorizedFunction, SorobanAuthorizedInvocation, Transaction,
};

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
            } // handle other variants
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
            } // handle other variants
        }
    }
}

#[derive(Clone)]
pub struct AuthorizedCallsForContract {
    pub contract_id: ContractId,
    pub remaining: u16,
}

impl AuthorizedCallsForContract {
    fn count_authorized_calls(&self, invocation: &SorobanAuthorizedInvocation) -> u16 {
        let mut count = 0;
        if let SorobanAuthorizedFunction::ContractFn(args) = &invocation.function {
            if args.contract_address.to_string() == self.contract_id.to_string() {
                count += 1;
            }
        }
        // visit all nodes in the tree of invocations.
        for sub_invocation in invocation.sub_invocations.iter() {
            count += self.count_authorized_calls(sub_invocation);
        }
        count
    }

    fn extract_contract_calls(&self, tx: &Transaction) -> u16 {
        let mut calls = 0;
        for op in tx.operations.iter() {
            if let OperationBody::InvokeHostFunction(invoke_op) = &op.body {
                for auth_entry in invoke_op.auth.iter() {
                    calls += self.count_authorized_calls(&auth_entry.root_invocation);
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

#[cfg(test)]
mod tests {
    use ed25519_dalek::SigningKey;
    use stellar_strkey::{Contract as ContractId, ed25519::PublicKey};
    use stellar_xdr::curr::{
        AccountId, Hash, HostFunction, InvokeContractArgs, InvokeHostFunctionOp, Operation,
        OperationBody, ScAddress, ScSymbol, SorobanAuthorizationEntry, SorobanAuthorizedFunction,
        SorobanAuthorizedInvocation, SorobanCredentials, VecM,
    };

    use crate::{
        Account, AuthorizedCallsForContract, Signer,
        mock::{mock_contract_id, mock_env, mock_transaction},
    };

    fn create_invocation(
        target_address: &ContractId,
        sub_invocations: Vec<SorobanAuthorizedInvocation>,
    ) -> SorobanAuthorizedInvocation {
        SorobanAuthorizedInvocation {
            function: SorobanAuthorizedFunction::ContractFn(InvokeContractArgs {
                contract_address: ScAddress::Contract(Hash(target_address.0)),
                function_name: ScSymbol("dummy_fn".try_into().unwrap()),
                args: VecM::default(),
            }),
            sub_invocations: sub_invocations.try_into().unwrap(),
        }
    }

    #[test]
    fn test_authorized_calls_check_and_update_success() {
        let signing_key = SigningKey::from_bytes(&[42; 32]);
        let public_key = PublicKey(*signing_key.verifying_key().as_bytes());
        let account_id = AccountId(stellar_xdr::curr::PublicKey::PublicKeyTypeEd25519(
            public_key.0.into(),
        ));
        let signer = Signer::new(signing_key);
        let account = Account::single(signer);
        let env = mock_env(None, None, None);

        let contract_id = mock_contract_id(account, &env);

        let sub_invocation = create_invocation(&contract_id, vec![]);
        let root_invocation = create_invocation(
            &contract_id,
            vec![sub_invocation.clone(), sub_invocation.clone()],
        );

        let auth_entry = SorobanAuthorizationEntry {
            credentials: SorobanCredentials::SourceAccount,
            root_invocation,
        };
        let invoke_op = InvokeHostFunctionOp {
            host_function: HostFunction::InvokeContract(InvokeContractArgs {
                contract_address: ScAddress::Contract(Hash(contract_id.0)),
                function_name: ScSymbol("dummy_fn".try_into().unwrap()),
                args: VecM::default(),
            }),
            auth: vec![auth_entry].try_into().unwrap(),
        };

        let op = Operation {
            source_account: None,
            body: OperationBody::InvokeHostFunction(invoke_op),
        };

        let transaction = mock_transaction(account_id.clone(), vec![op]);

        let mut guard = AuthorizedCallsForContract {
            contract_id,
            remaining: 3,
        };

        assert_eq!(guard.extract_contract_calls(&transaction), 3);
        assert!(guard.check(&transaction));
        guard.update(&transaction);
        assert_eq!(guard.remaining, 0);
    }

    #[test]
    fn test_authorized_calls_for_contract_check_and_update_fail() {
        let signing_key = SigningKey::from_bytes(&[42; 32]);
        let public_key = PublicKey(*signing_key.verifying_key().as_bytes());
        let account_id = AccountId(stellar_xdr::curr::PublicKey::PublicKeyTypeEd25519(
            public_key.0.into(),
        ));
        let signer = Signer::new(signing_key);
        let account = Account::single(signer);
        let env = mock_env(None, None, None);

        let contract_id = mock_contract_id(account, &env);

        let sub_invocation = create_invocation(&contract_id, vec![]);
        let root_invocation = create_invocation(&contract_id, vec![sub_invocation.clone()]);

        let auth_entry = SorobanAuthorizationEntry {
            credentials: SorobanCredentials::SourceAccount,
            root_invocation,
        };
        let invoke_op = InvokeHostFunctionOp {
            host_function: HostFunction::InvokeContract(InvokeContractArgs {
                contract_address: ScAddress::Contract(Hash(contract_id.0)),
                function_name: ScSymbol("dummy_fn".try_into().unwrap()),
                args: VecM::default(),
            }),
            auth: vec![auth_entry].try_into().unwrap(),
        };

        let op = Operation {
            source_account: None,
            body: OperationBody::InvokeHostFunction(invoke_op),
        };

        let transaction = mock_transaction(account_id.clone(), vec![op]);

        let guard = AuthorizedCallsForContract {
            contract_id,
            remaining: 1,
        };

        assert_eq!(guard.extract_contract_calls(&transaction), 2);
        assert!(!guard.check(&transaction));
    }
}
