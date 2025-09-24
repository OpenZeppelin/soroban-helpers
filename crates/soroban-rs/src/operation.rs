//! # Soroban Operation Creation
//!
//! This module provides functionality for creating Stellar operations for Soroban contracts.
//! These operations represent the fundamental actions that can be performed with Soroban,
//! such as uploading contract code, deploying contracts, and invoking contract functions.
use stellar_xdr::curr::{
    AccountId, Asset, ContractExecutable, ContractIdPreimage, CreateContractArgs,
    CreateContractArgsV2, Hash, HostFunction, InvokeContractArgs, InvokeHostFunctionOp, Operation,
    OperationBody, PaymentOp, ScAddress, ScSymbol, ScVal, SorobanAuthorizationEntry,
    SorobanAuthorizedFunction, SorobanAuthorizedInvocation, SorobanCredentials, VecM,
};

use crate::error::SorobanHelperError;

/// Factory for creating Soroban operations.
///
/// This struct provides methods to create operations for common Soroban tasks,
/// such as uploading contract WASM, deploying contracts, and invoking contract functions.
/// These operations can be added to transactions and submitted to the Stellar network.
pub struct Operations;

impl Operations {
    /// Creates an operation to upload contract WASM code to the Stellar network.
    ///
    /// # Parameters
    ///
    /// * `wasm_bytes` - The raw WASM bytecode to upload
    ///
    /// # Returns
    ///
    /// An operation that can be added to a transaction to upload the WASM
    ///
    /// # Errors
    ///
    /// Returns `SorobanHelperError::XdrEncodingFailed` if the WASM bytes
    /// cannot be encoded into the XDR format
    pub fn upload_wasm(wasm_bytes: Vec<u8>) -> Result<Operation, SorobanHelperError> {
        Ok(Operation {
            source_account: None,
            body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
                host_function: HostFunction::UploadContractWasm(wasm_bytes.try_into().map_err(
                    |e| {
                        SorobanHelperError::XdrEncodingFailed(format!(
                            "Failed to encode WASM bytes: {}",
                            e
                        ))
                    },
                )?),
                auth: VecM::default(),
            }),
        })
    }

    /// Creates an operation to deploy a contract to the Stellar network.
    ///
    /// # Parameters
    ///
    /// * `contract_id_preimage` - The preimage used to derive the contract ID
    /// * `wasm_hash` - The hash of the previously uploaded WASM code
    /// * `constructor_args` - Optional arguments to pass to the contract constructor
    ///
    /// # Returns
    ///
    /// An operation that can be added to a transaction to deploy the contract
    ///
    /// # Errors
    ///
    /// Returns `SorobanHelperError::XdrEncodingFailed` if any of the arguments
    /// cannot be encoded into the XDR format
    pub fn create_contract(
        contract_id_preimage: ContractIdPreimage,
        wasm_hash: Hash,
        constructor_args: Option<Vec<ScVal>>,
    ) -> Result<Operation, SorobanHelperError> {
        match constructor_args {
            Some(args) => {
                Self::create_contract_with_constructor(contract_id_preimage, wasm_hash, args)
            }
            None => Self::create_contract_without_constructor(contract_id_preimage, wasm_hash),
        }
    }

    /// Creates an operation to deploy a contract with constructor arguments.
    ///
    /// # Parameters
    ///
    /// * `contract_id_preimage` - The preimage used to derive the contract ID
    /// * `wasm_hash` - The hash of the previously uploaded WASM code
    /// * `constructor_args` - Arguments to pass to the contract constructor
    ///
    /// # Returns
    ///
    /// An operation that can be added to a transaction to deploy the contract
    ///
    /// # Errors
    ///
    /// Returns `SorobanHelperError::XdrEncodingFailed` if any of the arguments
    /// cannot be encoded into the XDR format
    fn create_contract_with_constructor(
        contract_id_preimage: ContractIdPreimage,
        wasm_hash: Hash,
        constructor_args: Vec<ScVal>,
    ) -> Result<Operation, SorobanHelperError> {
        let args: VecM<ScVal, { u32::MAX }> = constructor_args.try_into().map_err(|e| {
            SorobanHelperError::XdrEncodingFailed(format!(
                "Failed to encode constructor args: {}",
                e
            ))
        })?;

        let create_args = CreateContractArgsV2 {
            contract_id_preimage,
            executable: ContractExecutable::Wasm(wasm_hash),
            constructor_args: args,
        };

        let auth_entry = SorobanAuthorizationEntry {
            credentials: SorobanCredentials::SourceAccount,
            root_invocation: SorobanAuthorizedInvocation {
                function: SorobanAuthorizedFunction::CreateContractV2HostFn(create_args.clone()),
                sub_invocations: VecM::default(),
            },
        };

        Ok(Operation {
            source_account: None,
            body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
                auth: vec![auth_entry].try_into().map_err(|e| {
                    SorobanHelperError::XdrEncodingFailed(format!(
                        "Failed to encode auth entries: {}",
                        e
                    ))
                })?,
                host_function: HostFunction::CreateContractV2(create_args),
            }),
        })
    }

    /// Creates an operation to deploy a contract without constructor arguments.
    ///
    /// # Parameters
    ///
    /// * `contract_id_preimage` - The preimage used to derive the contract ID
    /// * `wasm_hash` - The hash of the previously uploaded WASM code
    ///
    /// # Returns
    ///
    /// An operation that can be added to a transaction to deploy the contract
    ///
    /// # Errors
    ///
    /// Returns `SorobanHelperError::XdrEncodingFailed` if any of the arguments
    /// cannot be encoded into the XDR format
    fn create_contract_without_constructor(
        contract_id_preimage: ContractIdPreimage,
        wasm_hash: Hash,
    ) -> Result<Operation, SorobanHelperError> {
        let create_args = CreateContractArgs {
            contract_id_preimage,
            executable: ContractExecutable::Wasm(wasm_hash),
        };

        let auth_entry = SorobanAuthorizationEntry {
            credentials: SorobanCredentials::SourceAccount,
            root_invocation: SorobanAuthorizedInvocation {
                function: SorobanAuthorizedFunction::CreateContractHostFn(create_args.clone()),
                sub_invocations: VecM::default(),
            },
        };

        Ok(Operation {
            source_account: None,
            body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
                auth: vec![auth_entry].try_into().map_err(|e| {
                    SorobanHelperError::XdrEncodingFailed(format!(
                        "Failed to encode auth entries: {}",
                        e
                    ))
                })?,
                host_function: HostFunction::CreateContract(create_args),
            }),
        })
    }

    /// Creates an operation to invoke a function on a deployed contract.
    ///
    /// # Parameters
    ///
    /// * `contract_id` - The ID of the deployed contract
    /// * `function_name` - The name of the function to invoke
    /// * `args` - Arguments to pass to the function
    ///
    /// # Returns
    ///
    /// An operation that can be added to a transaction to invoke the contract function
    ///
    /// # Errors
    ///
    /// Returns:
    /// - `SorobanHelperError::InvalidArgument` if the function name is invalid
    /// - `SorobanHelperError::XdrEncodingFailed` if the arguments cannot be encoded
    pub fn invoke_contract(
        contract_id: &stellar_strkey::Contract,
        function_name: &str,
        args: Vec<ScVal>,
    ) -> Result<Operation, SorobanHelperError> {
        let invoke_contract_args = InvokeContractArgs {
            contract_address: ScAddress::Contract(stellar_xdr::curr::ContractId(Hash(
                contract_id.0,
            ))),
            function_name: ScSymbol(function_name.try_into().map_err(|e| {
                SorobanHelperError::InvalidArgument(format!("Invalid function name: {}", e))
            })?),
            args: args.try_into().map_err(|e| {
                SorobanHelperError::XdrEncodingFailed(format!("Failed to encode arguments: {}", e))
            })?,
        };

        Ok(Operation {
            source_account: None,
            body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
                host_function: HostFunction::InvokeContract(invoke_contract_args),
                auth: VecM::default(),
            }),
        })
    }

    pub fn send_payment(
        to: AccountId,
        amount: i64,
        asset: Asset,
    ) -> Result<Operation, SorobanHelperError> {
        Ok(Operation {
            source_account: None,
            body: OperationBody::Payment(PaymentOp {
                amount,
                destination: to.into(),
                asset,
            }),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use stellar_xdr::curr::{ContractIdPreimageFromAddress, PublicKey, ScVal};

    #[test]
    fn test_upload_wasm() {
        let wasm_bytes = vec![0, 1, 2, 3, 4, 5];
        let operation = Operations::upload_wasm(wasm_bytes.clone()).unwrap();

        assert!(matches!(
            operation.body,
            OperationBody::InvokeHostFunction(_)
        ));
        if let OperationBody::InvokeHostFunction(op) = operation.body {
            assert!(matches!(
                op.host_function,
                HostFunction::UploadContractWasm(_)
            ));
            assert_eq!(op.auth.len(), 0);
        }
    }

    #[test]
    fn test_create_contract_without_args() {
        let account_id =
            stellar_xdr::curr::AccountId(PublicKey::PublicKeyTypeEd25519([0; 32].into()));
        let contract_id_preimage = ContractIdPreimage::Address(ContractIdPreimageFromAddress {
            address: ScAddress::Account(account_id),
            salt: [1; 32].into(),
        });

        let wasm_hash = Hash([2; 32]);
        let operation =
            Operations::create_contract(contract_id_preimage.clone(), wasm_hash.clone(), None)
                .unwrap();

        assert!(matches!(
            operation.body,
            OperationBody::InvokeHostFunction(_)
        ));
        if let OperationBody::InvokeHostFunction(op) = operation.body {
            assert!(matches!(op.host_function, HostFunction::CreateContract(_)));
            assert_eq!(op.auth.len(), 1);

            if let HostFunction::CreateContract(args) = op.host_function {
                assert_eq!(args.contract_id_preimage, contract_id_preimage);
                assert!(matches!(args.executable, ContractExecutable::Wasm(h) if h == wasm_hash));
            }
        }
    }

    #[test]
    fn test_create_contract_with_args() {
        let account_id =
            stellar_xdr::curr::AccountId(PublicKey::PublicKeyTypeEd25519([0; 32].into()));
        let contract_id_preimage = ContractIdPreimage::Address(ContractIdPreimageFromAddress {
            address: ScAddress::Account(account_id),
            salt: [1; 32].into(),
        });

        let wasm_hash = Hash([2; 32]);
        let constructor_args = vec![ScVal::I32(42), ScVal::Bool(true)];
        let operation = Operations::create_contract(
            contract_id_preimage.clone(),
            wasm_hash.clone(),
            Some(constructor_args.clone()),
        )
        .unwrap();

        assert!(matches!(
            operation.body,
            OperationBody::InvokeHostFunction(_)
        ));
        if let OperationBody::InvokeHostFunction(op) = operation.body {
            assert!(matches!(
                op.host_function,
                HostFunction::CreateContractV2(_)
            ));
            assert_eq!(op.auth.len(), 1);

            if let HostFunction::CreateContractV2(args) = op.host_function {
                assert_eq!(args.contract_id_preimage, contract_id_preimage);
                assert!(matches!(args.executable, ContractExecutable::Wasm(h) if h == wasm_hash));

                assert_eq!(args.constructor_args.len(), 2);
                assert!(matches!(args.constructor_args[0], ScVal::I32(42)));
                assert!(matches!(args.constructor_args[1], ScVal::Bool(true)));
            }
        }
    }

    #[test]
    fn test_invoke_contract() {
        let contract_bytes = [3; 32];
        let contract_id = stellar_strkey::Contract(contract_bytes);

        let function_name = "test_function";
        let args = vec![ScVal::I32(42), ScVal::Bool(true)];
        let operation =
            Operations::invoke_contract(&contract_id, function_name, args.clone()).unwrap();

        assert!(matches!(
            operation.body,
            OperationBody::InvokeHostFunction(_)
        ));
        if let OperationBody::InvokeHostFunction(op) = operation.body {
            assert!(matches!(op.host_function, HostFunction::InvokeContract(_)));
            assert_eq!(op.auth.len(), 0);

            if let HostFunction::InvokeContract(args) = op.host_function {
                assert!(
                    matches!(args.contract_address, ScAddress::Contract(stellar_xdr::curr::ContractId(hash)) if hash.0 == contract_bytes)
                );
                assert_eq!(args.function_name.0.as_slice(), function_name.as_bytes());

                assert_eq!(args.args.len(), 2);
                assert!(matches!(args.args[0], ScVal::I32(42)));
                assert!(matches!(args.args[1], ScVal::Bool(true)));
            }
        }
    }

    #[test]
    fn test_invoke_contract_invalid_function_name() {
        let contract_bytes = [3; 32];
        let contract_id = stellar_strkey::Contract(contract_bytes);

        let invalid_function_name = "a".repeat(33); // ScSymbol has a max length of 32
        let args = vec![];

        let result = Operations::invoke_contract(&contract_id, &invalid_function_name, args);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(SorobanHelperError::InvalidArgument(_))
        ));
    }
}
