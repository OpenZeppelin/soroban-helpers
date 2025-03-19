//! # Soroban Contract Management
//!
//! This module provides functionality for interacting with Soroban Smart Contracts,
//! including deployment and function invocation.
//!
//! ## Features
//!
//! - Loading contract WASM bytecode from file
//! - Deploying contracts to the Soroban network
//! - Invoking contract functions with arguments
//! - Managing contract identifiers
//!
//! ## Example
//!
//! ```rust,no_run
//! use soroban_rs::{Account, Contract, Env, EnvConfigs, Signer};
//! use stellar_xdr::curr::ScVal;
//!
//! async fn deploy_and_invoke() {
//!     // Setup environment and account
//!     let env = Env::new(...});
//!     let signing_key = SigningKey::from_bytes(...);
//!     let mut account = Account::single(Signer::new(signing_key));
//!
//!     // Load and deploy contract
//!     let contract = Contract::new("path/to/contract.wasm", None)?;
//!     let mut deployed = contract.deploy(&env, &mut account, None).await?;
//!
//!     // Invoke contract function
//!     let args = vec![/* function arguments as ScVal */];
//!     let result = deployed.invoke("function_name", args).await?;
//! }
//! ```
use crate::{
    crypto, error::SorobanHelperError, fs::{DefaultFileReader, FileReader}, operation::Operations, transaction::TransactionBuilder, Account, Env
};
use stellar_strkey::Contract as ContractId;
use stellar_xdr::curr::{
    ContractIdPreimage, ContractIdPreimageFromAddress, Hash, ScAddress, ScVal,
};

/// Name of the constructor function
const CONSTRUCTOR_FUNCTION_NAME: &str = "__constructor";

/// Configuration for client interaction with a deployed contract
///
/// Contains all necessary information to interact with a deployed contract,
/// including the contract identifier, environment, and signing account.
#[derive(Clone)]
pub struct ClientContractConfigs {
    /// The deployed contract's identifier
    pub contract_id: ContractId,
    /// The environment for interacting with the network
    pub env: Env,
    /// The account used for signing transactions
    pub account: Account,
}

/// Represents a Soroban smart contract
///
/// Provides functionality to deploy and interact with Soroban smart contracts.
/// A Contract instance can represent either an undeployed contract (with just WASM bytecode)
/// or a deployed contract (with client configuration for interacting with it).
pub struct Contract {
    /// Raw WASM bytecode of the contract
    wasm_bytes: Vec<u8>,
    /// SHA-256 hash of the WASM bytecode
    wasm_hash: Hash,
    /// Optional configuration for interacting with a deployed instance of this contract
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
    /// Creates a new Contract instance from a WASM file path
    ///
    /// # Parameters
    ///
    /// * `wasm_path` - Path to the contract's WASM file
    /// * `client_configs` - Optional configuration for interacting with an already deployed instance
    ///
    /// # Returns
    ///
    /// A new Contract instance or an error if the file couldn't be read
    pub fn new(
        wasm_path: &str,
        client_configs: Option<ClientContractConfigs>,
    ) -> Result<Self, SorobanHelperError> {
        Self::new_with_reader(wasm_path, client_configs, DefaultFileReader)
    }

    /// Creates a new Contract instance from a WASM file path and custom file reader
    ///
    /// ### Parameters
    ///
    /// * `wasm_path` - Path to the contract's WASM file
    /// * `client_configs` - Optional configuration for interacting with an already deployed instance
    /// * `file_reader` - Custom file reader for reading the WASM file adopting the `FileReader` trait.
    ///
    /// ### Returns
    ///
    /// A new Contract instance or an error if the file couldn't be read
    pub fn new_with_reader<T: FileReader>(
        wasm_path: &str,
        client_configs: Option<ClientContractConfigs>,
        file_reader: T,
    ) -> Result<Self, SorobanHelperError> {
        let wasm_bytes = file_reader.read(wasm_path)?;
        let wasm_hash = crypto::sha256_hash(&wasm_bytes);

        Ok(Self {
            wasm_bytes,
            wasm_hash,
            client_configs,
        })
    }

    /// Deploys the contract to the Soroban network
    ///
    /// This method performs two operations:
    /// 1. Uploads the contract WASM bytecode if it doesn't exist on the network
    /// 2. Creates a contract instance with the uploaded WASM
    ///
    /// If the contract has a constructor function,
    /// the provided constructor arguments will be passed to it during deployment.
    ///
    /// # Parameters
    ///
    /// * `env` - The environment to use for deployment
    /// * `account` - The account that will deploy the contract and pay for the transaction
    /// * `constructor_args` - Optional arguments to pass to the contract's constructor
    ///
    /// # Returns
    ///
    /// The Contract instance updated with client configuration for the deployed contract
    pub async fn deploy(
        mut self,
        env: &Env,
        account: &mut Account,
        constructor_args: Option<Vec<ScVal>>,
    ) -> Result<Self, SorobanHelperError> {
        self.upload_wasm(account, env).await?;

        let salt = crypto::generate_salt();
        let contract_id =
            crypto::calculate_contract_id(&account.account_id(), &salt, &env.network_id())?;

        let contract_id_preimage = ContractIdPreimage::Address(ContractIdPreimageFromAddress {
            address: ScAddress::Account(account.account_id()),
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

        let builder = TransactionBuilder::new(account, env).add_operation(create_operation);

        let deploy_tx = builder.simulate_and_build(env, account).await?;
        let tx_envelope = account.sign_transaction(&deploy_tx, &env.network_id())?;
        env.send_transaction(&tx_envelope).await?;

        self.set_client_configs(ClientContractConfigs {
            contract_id,
            env: env.clone(),
            account: account.clone(),
        });

        Ok(self)
    }

    /// Sets the client configuration for interacting with a deployed contract
    ///
    /// # Parameters
    ///
    /// * `client_configs` - The client configuration to set
    fn set_client_configs(&mut self, client_configs: ClientContractConfigs) {
        self.client_configs = Some(client_configs);
    }

    /// Returns the contract ID if the contract has been deployed
    ///
    /// # Returns
    ///
    /// The contract ID or None if the contract has not been deployed
    pub fn contract_id(&self) -> Option<ContractId> {
        self.client_configs.as_ref().map(|c| c.contract_id)
    }

    /// Uploads the contract WASM bytecode to the network
    ///
    /// # Parameters
    ///
    /// * `account` - The account that will pay for the upload
    /// * `env` - The environment to use for the upload
    ///
    /// # Returns
    ///
    /// Ok(()) if the upload was successful or the code already exists
    async fn upload_wasm(
        &self,
        account: &mut Account,
        env: &Env,
    ) -> Result<(), SorobanHelperError> {
        let upload_operation = Operations::upload_wasm(self.wasm_bytes.clone())?;

        let builder = TransactionBuilder::new(account, env).add_operation(upload_operation);

        let upload_tx = builder.simulate_and_build(env, account).await?;
        let tx_envelope = account.sign_transaction(&upload_tx, &env.network_id())?;

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

    /// Invokes a function on the deployed contract
    ///
    /// # Parameters
    ///
    /// * `function_name` - The name of the function to invoke
    /// * `args` - The arguments to pass to the function
    ///
    /// # Returns
    ///
    /// The transaction response from the network
    ///
    /// # Errors
    ///
    /// Returns an error if the contract has not been deployed or
    /// if there's an issue with the invocation
    pub async fn invoke(
        &mut self,
        function_name: &str,
        args: Vec<ScVal>,
    ) -> Result<stellar_rpc_client::GetTransactionResponse, SorobanHelperError> {
        let client_configs = self
            .client_configs
            .as_mut()
            .ok_or(SorobanHelperError::ContractDeployedConfigsNotSet)?;

        let contract_id = client_configs.contract_id;
        let env = client_configs.env.clone();

        let invoke_operation = Operations::invoke_contract(&contract_id, function_name, args)?;

        let builder =
            TransactionBuilder::new(&client_configs.account, &env).add_operation(invoke_operation);

        let invoke_tx = builder
            .simulate_and_build(&env, &client_configs.account)
            .await?;
        let tx_envelope = client_configs
            .account
            .sign_transaction(&invoke_tx, &env.network_id())?;

        env.send_transaction(&tx_envelope).await
    }
}

#[cfg(test)]
mod test {
    use crate::{ mock::fs::MockFileReader, Contract};

    #[tokio::test]
    async fn test_file_reader() {
        let wasm_path = "path/to/wasm";
        let client_configs = None;
        let file_reader = MockFileReader::new(Ok(b"mock wasm bytes".to_vec()));
        let contract = Contract::new_with_reader(wasm_path, client_configs, file_reader).unwrap();
        assert_eq!(contract.wasm_bytes, b"mock wasm bytes".to_vec());
    }


    #[tokio::test]
    async fn test_upload_wasm() {
        // TODO.
    }

    #[tokio::test]
    async fn test_contract_deploy() {
        // TODO.
    }

    #[tokio::test]
    async fn test_contract_invoke() {
        // TODO.
    }

}
