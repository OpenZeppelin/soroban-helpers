use std::fs;
use rand;
use sha2::{Digest, Sha256};
use stellar_xdr::curr::{
    ContractExecutable, ContractIdPreimage, ContractIdPreimageFromAddress, CreateContractArgs,
    CreateContractArgsV2, Hash, HashIdPreimage, HashIdPreimageContractId, HostFunction,
    InvokeHostFunctionOp, Memo, Operation, OperationBody, Preconditions, ScAddress, ScVal,
    SequenceNumber, SorobanAuthorizationEntry, SorobanAuthorizedFunction, SorobanAuthorizedInvocation,
    SorobanCredentials, Transaction, TransactionExt, Uint256, VecM, WriteXdr, Limits
};
use crate::{Provider, Signer};

const DEFAULT_TRANSACTION_FEES: u32 = 100;
const CONSTRUCTOR_FUNCTION_NAME: &str = "__constructor";

pub struct Contract {
    wasm_path: String,
    wasm_bytes: Vec<u8>,
    wasm_hash: Hash,
}

impl Contract {
    pub fn new(wasm_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let wasm_bytes = fs::read(wasm_path)?;
        let wasm_hash_bytes: [u8; 32] = Sha256::digest(&wasm_bytes).into();
        let wasm_hash = Hash(wasm_hash_bytes);
        
        Ok(Self {
            wasm_path: wasm_path.to_string(),
            wasm_bytes,
            wasm_hash,
        })
    }
    
    pub async fn deploy(
        &self,
        provider: &Provider,
        signer: &mut Signer,
        constructor_args: Option<Vec<ScVal>>,
    ) -> Result<stellar_strkey::Contract, Box<dyn std::error::Error>> {
        
        // Then deploy the contract
        let account_id = signer.account_id().clone();
        let account_details = provider.get_account(&account_id.to_string()).await?;
        let sequence: i64 = account_details.seq_num.into();

        // First upload the WASM
        self.upload_wasm(provider, signer, sequence + 1).await?;
        
        // Create a unique salt
        let salt_bytes: [u8; 32] = rand::random();
        let salt = Uint256(salt_bytes);
        
        // Create contract ID preimage
        let contract_id_preimage = ContractIdPreimage::Address(ContractIdPreimageFromAddress {
            address: ScAddress::Account(account_id.clone()),
            salt,
        });
        
        // Calculate contract ID
        let preimage = HashIdPreimage::ContractId(HashIdPreimageContractId {
            network_id: provider.network_id().clone(),
            contract_id_preimage: contract_id_preimage.clone(),
        });
        
        let preimage_xdr = preimage.to_xdr(Limits::none())?;
        let contract_id = stellar_strkey::Contract(Sha256::digest(preimage_xdr).into());
        
        // Check if contract has constructor
        let has_constructor = String::from_utf8_lossy(&self.wasm_bytes).contains(CONSTRUCTOR_FUNCTION_NAME);
        
        // Create deployment operation
        let create_operation = if has_constructor && constructor_args.is_some() {
            // Use V2 with constructor args
            let args: VecM<ScVal, { u32::MAX }> = constructor_args.unwrap_or_default().try_into()?;
            
            let create_args = CreateContractArgsV2 {
                contract_id_preimage: contract_id_preimage.clone(),
                executable: ContractExecutable::Wasm(self.wasm_hash.clone()),
                constructor_args: args,
            };
            
            let auth_entry = SorobanAuthorizationEntry {
                credentials: SorobanCredentials::SourceAccount,
                root_invocation: SorobanAuthorizedInvocation {
                    function: SorobanAuthorizedFunction::CreateContractV2HostFn(create_args.clone()),
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
            // Use V1 (no constructor or no args)
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
        
        let operations: VecM<_, 100> = vec![create_operation].try_into()?;
        
        let mut deploy_tx = Transaction {
            fee: DEFAULT_TRANSACTION_FEES,
            seq_num: SequenceNumber(sequence + 2),
            source_account: account_id.into(),
            cond: Preconditions::None,
            memo: Memo::None,
            operations,
            ext: TransactionExt::V0,
        };
        
        // Simulate transaction
        let tx_envelope = signer.sign_transaction(&deploy_tx, provider.network_id())?;
        let simulation = provider.simulate_transaction(&tx_envelope).await?;
        
        // Update fee and transaction data
        deploy_tx.fee = deploy_tx.fee.max(
            u32::try_from(DEFAULT_TRANSACTION_FEES as u64 + simulation.min_resource_fee)
                .expect("Transaction fee too high"),
        );
        
        if let Ok(tx_data) = simulation.transaction_data() {
            deploy_tx.ext = TransactionExt::V1(tx_data);
        }
        
        // Sign and submit
        let tx_envelope = signer.sign_transaction(&deploy_tx, provider.network_id())?;
        provider.send_transaction(&tx_envelope).await?;
        
        Ok(contract_id)
    }
    
    async fn upload_wasm(
        &self,
        provider: &Provider,
        signer: &mut Signer,
        sequence: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let account_id = signer.account_id().clone();
        
        let upload_operation = Operation {
            source_account: None,
            body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
                host_function: HostFunction::UploadContractWasm(self.wasm_bytes.clone().try_into()?),
                auth: VecM::default(),
            }),
        };
        
        let operations: VecM<_, 100> = vec![upload_operation].try_into()?;
        
        let mut upload_tx = Transaction {
            fee: DEFAULT_TRANSACTION_FEES,
            seq_num: SequenceNumber(sequence),
            source_account: account_id.into(),
            cond: Preconditions::None,
            memo: Memo::None,
            operations,
            ext: TransactionExt::V0,
        };
        
        // Simulate transaction
        let tx_envelope = signer.sign_transaction(&upload_tx, provider.network_id())?;
        let simulation = provider.simulate_transaction(&tx_envelope).await?;
        
        // Update fee and transaction data
        upload_tx.fee = upload_tx.fee.max(
            u32::try_from(DEFAULT_TRANSACTION_FEES as u64 + simulation.min_resource_fee)
                .expect("Transaction fee too high"),
        );
        
        if let Ok(tx_data) = simulation.transaction_data() {
            upload_tx.ext = TransactionExt::V1(tx_data);
        }
        
        // Sign and submit
        let tx_envelope = signer.sign_transaction(&upload_tx, provider.network_id())?;
        
        match provider.send_transaction(&tx_envelope).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // If it failed because the code already exists, that's fine
                if e.to_string().contains("ContractCodeAlreadyExists") {
                    Ok(())
                } else {
                    Err(Box::new(e))
                }
            }
        }
    }
    
    // Add methods to interact with the deployed contract
    pub async fn call(
        &self,
        contract_id: &stellar_strkey::Contract,
        function_name: &str,
        args: Vec<ScVal>,
        provider: &Provider,
        signer: &mut Signer,
    ) -> Result<Vec<ScVal>, Box<dyn std::error::Error>> {
        // Implementation for calling contract functions
        // This would create a transaction to invoke the contract function
        // Similar to the deploy method but using InvokeContract
        todo!("Implement contract function calling")
    }
}