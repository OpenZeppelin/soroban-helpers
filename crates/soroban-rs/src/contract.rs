use crate::{
    Account, Env, crypto, error::SorobanHelperError, operation::Operations, parser,
    transaction::TransactionBuilder,
};
use std::fs;
use stellar_strkey::Contract as ContractId;
use stellar_xdr::curr::{
    ContractIdPreimage, ContractIdPreimageFromAddress, Hash, ScAddress, ScVal,
};

const CONSTRUCTOR_FUNCTION_NAME: &str = "__constructor";

pub struct ClientContractConfigs {
    pub contract_id: ContractId,
    pub env: Env,
    pub account: Account,
}

impl Clone for ClientContractConfigs {
    fn clone(&self) -> Self {
        Self {
            contract_id: self.contract_id,
            env: self.env.clone(),
            account: self.account.clone(),
        }
    }
}

pub struct Contract {
    wasm_bytes: Vec<u8>,
    wasm_hash: Hash,
    client_configs: Option<ClientContractConfigs>,
}

impl Clone for Contract {
    fn clone(&self) -> Self {
        Self {
            wasm_bytes: self.wasm_bytes.clone(),
            wasm_hash: self.wasm_hash.clone(),
            client_configs: self.client_configs.clone(),
        }
    }
}

impl Contract {
    pub fn new(
        wasm_path: &str,
        client_configs: Option<ClientContractConfigs>,
    ) -> Result<Self, SorobanHelperError> {
        let wasm_bytes = fs::read(wasm_path)?;
        let wasm_hash = crypto::sha256_hash(&wasm_bytes);

        Ok(Self {
            wasm_bytes,
            wasm_hash,
            client_configs,
        })
    }

    pub async fn deploy(
        mut self,
        env: &Env,
        account: &mut Account,
        constructor_args: Option<Vec<ScVal>>,
    ) -> Result<Self, SorobanHelperError> {
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

        self.set_client_configs(ClientContractConfigs {
            contract_id,
            env: env.clone(),
            account: account.clone(),
        });

        Ok(self)
    }

    fn set_client_configs(&mut self, client_configs: ClientContractConfigs) {
        self.client_configs = Some(client_configs);
    }

    pub fn contract_id(&self) -> Option<ContractId> {
        self.client_configs.as_ref().map(|c| c.contract_id)
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
        &mut self,
        function_name: &str,
        args: Vec<ScVal>,
    ) -> Result<ScVal, SorobanHelperError> {
        let client_configs = self
            .client_configs
            .as_mut()
            .ok_or(SorobanHelperError::ContractDeployedConfigsNotSet)?;

        let contract_id = client_configs.contract_id;
        let env = client_configs.env.clone();

        let sequence = client_configs.account.get_sequence(&env).await?;
        let account_id = client_configs.account.account_id();

        let invoke_operation = Operations::invoke_contract(&contract_id, function_name, args)?;

        let builder =
            TransactionBuilder::new(account_id, sequence.0 + 1).add_operation(invoke_operation);

        let invoke_tx = builder
            .simulate_and_build(&env, &client_configs.account)
            .await?;
        let tx_envelope = client_configs
            .account
            .sign_transaction(&invoke_tx, env.network_id())?;
        let result = env.send_transaction(&tx_envelope).await?;

        parser::parse_transaction_result(&result)
    }
}
