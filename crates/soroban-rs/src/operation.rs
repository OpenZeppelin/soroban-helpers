use stellar_xdr::curr::{
    ContractExecutable, ContractIdPreimage, CreateContractArgs, CreateContractArgsV2, Hash,
    HostFunction, InvokeContractArgs, InvokeHostFunctionOp, Operation, OperationBody, ScAddress,
    ScSymbol, ScVal, SorobanAuthorizationEntry, SorobanAuthorizedFunction,
    SorobanAuthorizedInvocation, SorobanCredentials, VecM,
};

use crate::error::SorobanHelperError;

pub struct Operations;

impl Operations {
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

    pub fn invoke_contract(
        contract_id: &stellar_strkey::Contract,
        function_name: &str,
        args: Vec<ScVal>,
    ) -> Result<Operation, SorobanHelperError> {
        let invoke_contract_args = InvokeContractArgs {
            contract_address: ScAddress::Contract(Hash(contract_id.0)),
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
}
