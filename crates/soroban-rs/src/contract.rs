use crate::{
    Account, Env, crypto, error::SorobanHelperError, operation::Operations, parser,
    transaction::TransactionBuilder,
};
use std::fs;
use stellar_xdr::curr::{
    ContractIdPreimage, ContractIdPreimageFromAddress, Hash, ScAddress, ScVal,
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
        env: &Env,
        account: &mut Account,
        constructor_args: Option<Vec<ScVal>>,
    ) -> Result<stellar_strkey::Contract, SorobanHelperError> {
        let sequence = account.get_sequence(env).await?;
        let account_id = account.account_id();

        self.upload_wasm(env, account, sequence.0 + 1).await?;

        let salt = crypto::generate_salt();
        let contract_id = crypto::calculate_contract_id(&account_id, &salt, env.network_id())?;

        let contract_id_preimage = ContractIdPreimage::Address(ContractIdPreimageFromAddress {
            address: ScAddress::Account(account_id.clone()),
            salt,
        });

        let has_constructor =
            String::from_utf8_lossy(&self.wasm_bytes).contains(CONSTRUCTOR_FUNCTION_NAME);
        let create_operation = Operations::create_contract(
            contract_id_preimage,
            self.wasm_hash.clone(),
            if has_constructor {
                constructor_args
            } else {
                None
            },
        )?;

        let builder =
            TransactionBuilder::new(account_id, sequence.0 + 2).add_operation(create_operation);

        let deploy_tx = builder.simulate_and_build(env, account).await?;
        let tx_envelope = account.sign_transaction(&deploy_tx, env.network_id())?;
        env.send_transaction(&tx_envelope).await?;

        Ok(contract_id)
    }

    async fn upload_wasm(
        &self,
        env: &Env,
        account: &mut Account,
        sequence_num: i64,
    ) -> Result<(), SorobanHelperError> {
        let upload_operation = Operations::upload_wasm(self.wasm_bytes.clone())?;

        let builder = TransactionBuilder::new(account.account_id(), sequence_num)
            .add_operation(upload_operation);

        let upload_tx = builder.simulate_and_build(env, account).await?;
        let tx_envelope = account.sign_transaction(&upload_tx, env.network_id())?;

        match env.send_transaction(&tx_envelope).await {
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
        env: &Env,
        account: &mut Account,
    ) -> Result<ScVal, SorobanHelperError> {
        let sequence = account.get_sequence(env).await?;
        let account_id = account.account_id();

        let invoke_operation = Operations::invoke_contract(contract_id, function_name, args)?;

        let builder =
            TransactionBuilder::new(account_id, sequence.0 + 1).add_operation(invoke_operation);

        let invoke_tx = builder.simulate_and_build(env, account).await?;
        let tx_envelope = account.sign_transaction(&invoke_tx, env.network_id())?;
        let result = env.send_transaction(&tx_envelope).await?;

        parser::parse_transaction_result(&result)
    }
}
