use crate::{
    Provider, Signer, account::AccountManager, crypto, parser, transaction::TransactionBuilder,
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
    pub fn new(wasm_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
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
        signer: &Signer,
        constructor_args: Option<Vec<ScVal>>,
    ) -> Result<stellar_strkey::Contract, Box<dyn std::error::Error>> {
        let account_manager = AccountManager::new(provider, signer);
        let sequence = account_manager.get_sequence().await?;
        let account_id = account_manager.account_id().clone();

        self.upload_wasm(provider, signer, sequence + 1).await?;

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
            let args: VecM<ScVal, { u32::MAX }> =
                constructor_args.unwrap_or_default().try_into()?;

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
                    auth: vec![auth_entry].try_into()?,
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
                    auth: vec![auth_entry].try_into()?,
                    host_function: HostFunction::CreateContract(create_args),
                }),
            }
        };

        let mut builder = TransactionBuilder::new(account_id.into(), sequence + 2);
        builder.add_operation(create_operation);

        let deploy_tx = builder.simulate_and_build(provider, signer).await?;

        let tx_envelope = account_manager.sign_transaction(&deploy_tx)?;
        account_manager.send_transaction(&tx_envelope).await?;

        Ok(contract_id)
    }

    async fn upload_wasm(
        &self,
        provider: &Provider,
        signer: &Signer,
        sequence: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let account_manager = AccountManager::new(provider, signer);
        let account_id = account_manager.account_id().clone();

        let upload_operation = Operation {
            source_account: None,
            body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
                host_function: HostFunction::UploadContractWasm(
                    self.wasm_bytes.clone().try_into()?,
                ),
                auth: VecM::default(),
            }),
        };

        let mut builder = TransactionBuilder::new(account_id.into(), sequence);
        builder.add_operation(upload_operation);

        let upload_tx = builder.simulate_and_build(provider, signer).await?;
        let tx_envelope = account_manager.sign_transaction(&upload_tx)?;

        match account_manager.send_transaction(&tx_envelope).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // If it failed because the code already exists, that's fine
                if e.to_string().contains("ContractCodeAlreadyExists") {
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
        signer: &mut Signer,
    ) -> Result<ScVal, Box<dyn std::error::Error>> {
        let account_manager = AccountManager::new(provider, signer);
        let sequence = account_manager.get_sequence().await?;
        let account_id = account_manager.account_id().clone();

        let invoke_contract_args = InvokeContractArgs {
            contract_address: ScAddress::Contract(Hash(contract_id.0)),
            function_name: ScSymbol(function_name.try_into().unwrap()),
            args: args.try_into()?,
        };

        let invoke_operation = Operation {
            source_account: None,
            body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
                host_function: HostFunction::InvokeContract(invoke_contract_args),
                auth: VecM::default(),
            }),
        };

        let mut builder = TransactionBuilder::new(account_id.into(), sequence + 1);
        builder.add_operation(invoke_operation);

        let invoke_tx = builder.simulate_and_build(provider, signer).await?;
        let tx_envelope = account_manager.sign_transaction(&invoke_tx)?;
        let result = account_manager.send_transaction(&tx_envelope).await?;

        parser::parse_transaction_result(&result)
    }
}
