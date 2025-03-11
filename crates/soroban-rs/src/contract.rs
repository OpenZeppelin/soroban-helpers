use crate::{
    crypto, error::SorobanHelperError, parser, transaction::TransactionBuilder, Account, Provider
};
use std::fs;
use stellar_xdr::curr::{
    ContractExecutable, ContractIdPreimage, ContractIdPreimageFromAddress, CreateContractArgs,
    CreateContractArgsV2, Hash, HostFunction, InvokeContractArgs, InvokeHostFunctionOp, Operation,
    OperationBody, ScAddress, ScSymbol, ScVal, SorobanAuthorizationEntry,
    SorobanAuthorizedFunction, SorobanAuthorizedInvocation, SorobanCredentials, VecM,
};

const CONSTRUCTOR_FUNCTION_NAME: &str = "__constructor";

pub struct Contract {
    wasm_bytes: Vec<u8>,
    wasm_hash: Hash,
}

impl Contract {
    pub fn new(wasm_path: &str) -> Result<Self, SorobanHelperError> {
        let wasm_bytes = fs::read(wasm_path)?;
        let wasm_hash = crypto::sha256_hash(&wasm_bytes);

        Ok(Self {
            wasm_bytes,
            wasm_hash,
        })
    }

    pub async fn deploy(
        &self,
        provider: &Provider,
        account: &Account,
        constructor_args: Option<Vec<ScVal>>,
    ) -> Result<stellar_strkey::Contract, SorobanHelperError> {
        let sequence = account.get_sequence(provider).await?;
        let account_id = account.account_id();

        self.upload_wasm(provider, account, sequence.0 + 1).await?;

        let salt = crypto::generate_salt();

        let contract_id = crypto::calculate_contract_id(&account_id, &salt, provider.network_id())?;

        let contract_id_preimage = ContractIdPreimage::Address(ContractIdPreimageFromAddress {
            address: ScAddress::Account(account_id.clone()),
            salt,
        });

        let has_constructor =
            String::from_utf8_lossy(&self.wasm_bytes).contains(CONSTRUCTOR_FUNCTION_NAME);

        let create_operation = if has_constructor && constructor_args.is_some() {
            // Use V2 with constructor args
            let args: VecM<ScVal, { u32::MAX }> = constructor_args
                .unwrap_or_default()
                .try_into()
                .map_err(|e| {
                SorobanHelperError::XdrEncodingFailed(format!(
                    "Failed to encode constructor args: {}",
                    e
                ))
            })?;

            let create_args = CreateContractArgsV2 {
                contract_id_preimage: contract_id_preimage.clone(),
                executable: ContractExecutable::Wasm(self.wasm_hash.clone()),
                constructor_args: args,
            };

            let auth_entry = SorobanAuthorizationEntry {
                credentials: SorobanCredentials::SourceAccount,
                root_invocation: SorobanAuthorizedInvocation {
                    function: SorobanAuthorizedFunction::CreateContractV2HostFn(
                        create_args.clone(),
                    ),
                    sub_invocations: VecM::default(),
                },
            };

            Operation {
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
            }
        } else {
            let create_args = CreateContractArgs {
                contract_id_preimage: contract_id_preimage.clone(),
                executable: ContractExecutable::Wasm(self.wasm_hash.clone()),
            };

            let auth_entry = SorobanAuthorizationEntry {
                credentials: SorobanCredentials::SourceAccount,
                root_invocation: SorobanAuthorizedInvocation {
                    function: SorobanAuthorizedFunction::CreateContractHostFn(create_args.clone()),
                    sub_invocations: VecM::default(),
                },
            };

            Operation {
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
            }
        };

        let mut builder = TransactionBuilder::new(account_id.into(), sequence.0 + 2);
        builder.add_operation(create_operation);

        let deploy_tx = builder.simulate_and_build(provider, account).await?;

        let tx_envelope = account.sign_transaction(&deploy_tx, provider.network_id())?;
        provider.send_transaction(&tx_envelope).await?;

        Ok(contract_id)
    }

    async fn upload_wasm(
        &self,
        provider: &Provider,
        account: &Account,
        sequence_num: i64,
    ) -> Result<(), SorobanHelperError> {
        let upload_operation = Operation {
            source_account: None,
            body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
                host_function: HostFunction::UploadContractWasm(
                    self.wasm_bytes.clone().try_into().map_err(|e| {
                        SorobanHelperError::XdrEncodingFailed(format!(
                            "Failed to encode WASM bytes: {}",
                            e
                        ))
                    })?,
                ),
                auth: VecM::default(),
            }),
        };

        let mut builder = TransactionBuilder::new(account.account_id().into(), sequence_num);
        builder.add_operation(upload_operation);

        let upload_tx = builder.simulate_and_build(provider, account).await?;
        let tx_envelope = account.sign_transaction(&upload_tx, provider.network_id())?;

        match provider.send_transaction(&tx_envelope).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // If it failed because the code already exists, that's fine
                if let SorobanHelperError::ContractCodeAlreadyExists = e {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    pub async fn invoke(
        &self,
        contract_id: &stellar_strkey::Contract,
        function_name: &str,
        args: Vec<ScVal>,
        provider: &Provider,
        account: &Account,
    ) -> Result<ScVal, SorobanHelperError> {
        let sequence = account.get_sequence(provider).await?;
        let account_id = account.account_id();

        let invoke_contract_args = InvokeContractArgs {
            contract_address: ScAddress::Contract(Hash(contract_id.0)),
            function_name: ScSymbol(function_name.try_into().map_err(|e| {
                SorobanHelperError::InvalidArgument(format!("Invalid function name: {}", e))
            })?),
            args: args.try_into().map_err(|e| {
                SorobanHelperError::XdrEncodingFailed(format!("Failed to encode arguments: {}", e))
            })?,
        };

        let invoke_operation = Operation {
            source_account: None,
            body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
                host_function: HostFunction::InvokeContract(invoke_contract_args),
                auth: VecM::default(),
            }),
        };

        let mut builder = TransactionBuilder::new(
            account_id.into(),
            sequence.0 + 1
        );
        builder.add_operation(invoke_operation);

        let invoke_tx = builder.simulate_and_build(provider, account).await?;
        let tx_envelope = account.sign_transaction(&invoke_tx, provider.network_id())?;
        let result = provider.send_transaction(&tx_envelope).await?;

        parser::parse_transaction_result(&result)
    }
}
