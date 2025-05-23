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
//! use ed25519_dalek::SigningKey;
//!
//! async fn deploy_and_invoke() {
//!     // Setup environment and account
//!     let env = Env::new(EnvConfigs {
//!         rpc_url: "https://soroban-testnet.stellar.org".to_string(),
//!         network_passphrase: "Test SDF Network ; September 2015".to_string(),
//!     }).unwrap();
//!
//!     let private_key_bytes: [u8; 32] = [
//!         1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
//!         26, 27, 28, 29, 30, 31, 32,
//!     ];
//!     let signing_key = SigningKey::from_bytes(&private_key_bytes);
//!     let mut account = Account::single(Signer::new(signing_key));
//!
//!     // Load and deploy contract
//!     let contract = Contract::new("path/to/contract.wasm", None).unwrap();
//!     let mut deployed = contract.deploy(&env, &mut account, None).await.unwrap();
//!
//!     // Invoke contract function
//!     let args = vec![/* function arguments as ScVal */];
//!     let result = deployed.invoke("function_name", args).await.unwrap();
//! }
//! ```
use crate::{
    crypto,
    error::SorobanHelperError,
    fs::{DefaultFileReader, FileReader},
    operation::Operations,
    transaction::TransactionBuilder,
    Account, Env, ParseResult, Parser, ParserType, SorobanTransactionResponse,
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
    pub source_account: Account,
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

    /// Creates a new Contract instance from a deployed contract
    ///
    /// # Parameters
    ///
    /// * `client_configs` - Configuration for interacting with the deployed contract
    ///
    /// # Returns
    ///
    /// A new Contract instance
    pub fn from_configs(client_configs: ClientContractConfigs) -> Self {
        Self {
            wasm_bytes: Vec::new(),
            wasm_hash: crypto::sha256_hash(&[]),
            client_configs: Some(client_configs),
        }
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
        let tx_result = env.send_transaction(&tx_envelope).await?;

        let parser = Parser::new(ParserType::Deploy);
        let result = parser.parse(&tx_result.response)?;

        let contract_id = match result {
            ParseResult::Deploy(Some(contract_id)) => contract_id,
            _ => return Err(SorobanHelperError::ContractDeployedConfigsNotSet),
        };

        self.set_client_configs(ClientContractConfigs {
            contract_id,
            env: env.clone(),
            source_account: account.clone(),
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
    ) -> Result<SorobanTransactionResponse, SorobanHelperError> {
        let client_configs = self
            .client_configs
            .as_mut()
            .ok_or(SorobanHelperError::ContractDeployedConfigsNotSet)?;

        let contract_id = client_configs.contract_id;
        let env = client_configs.env.clone();

        let invoke_operation = Operations::invoke_contract(&contract_id, function_name, args)?;

        let builder = TransactionBuilder::new(&client_configs.source_account, &env)
            .add_operation(invoke_operation);

        let invoke_tx = builder
            .simulate_and_build(&env, &client_configs.source_account)
            .await?;

        let tx_envelope = client_configs
            .source_account
            .sign_transaction(&invoke_tx, &env.network_id())?;

        env.send_transaction(&tx_envelope).await
    }
}

#[cfg(test)]
mod test {
    use crate::{
        crypto,
        error::SorobanHelperError,
        mock::{
            fs::MockFileReader,
            mock_account_entry, mock_contract_id, mock_env, mock_signer1,
            mock_simulate_tx_response, mock_transaction_response,
            transaction::{create_contract_id_val, mock_transaction_response_with_return_value},
        },
        Account, ClientContractConfigs, Contract,
    };
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_contract_clone() {
        let wasm_bytes = b"mock wasm bytes".to_vec();
        let wasm_hash = crypto::sha256_hash(&wasm_bytes);
        let env = mock_env(None, None, None);
        let account = Account::single(mock_signer1());

        let client_configs = Some(ClientContractConfigs {
            contract_id: mock_contract_id(account.clone(), &env),
            env: env.clone(),
            source_account: account.clone(),
        });

        let original_contract = Contract {
            wasm_bytes: wasm_bytes.clone(),
            wasm_hash,
            client_configs: client_configs.clone(),
        };

        let cloned_contract = original_contract.clone();

        assert_eq!(cloned_contract.wasm_bytes, original_contract.wasm_bytes);
        assert_eq!(cloned_contract.wasm_hash.0, original_contract.wasm_hash.0);

        assert!(cloned_contract.client_configs.is_some());
        let cloned_configs = cloned_contract.client_configs.unwrap();
        let original_configs = original_contract.client_configs.unwrap();

        assert_eq!(cloned_configs.contract_id.0, original_configs.contract_id.0);
    }

    #[test]
    fn test_contract_new() {
        // Create fake temp wasm file because of DefaultFileReader
        let mut temp_file = NamedTempFile::new().unwrap();
        let wasm_bytes = b"test wasm bytes";
        temp_file.write_all(wasm_bytes).unwrap();

        let wasm_path = temp_file.path().to_str().unwrap();
        let contract = Contract::new(wasm_path, None).unwrap();

        assert_eq!(contract.wasm_bytes, wasm_bytes);
        assert_eq!(contract.wasm_hash, crypto::sha256_hash(wasm_bytes));
        assert!(contract.client_configs.is_none());
    }

    #[test]
    fn test_contract_id() {
        let wasm_bytes = b"mock wasm bytes".to_vec();
        let contract_without_configs = Contract {
            wasm_bytes: wasm_bytes.clone(),
            wasm_hash: crypto::sha256_hash(&wasm_bytes),
            client_configs: None,
        };

        assert!(contract_without_configs.contract_id().is_none());

        let env = mock_env(None, None, None);
        let account = Account::single(mock_signer1());
        let contract_id = mock_contract_id(account.clone(), &env);

        let contract_with_configs = Contract {
            wasm_bytes: wasm_bytes.clone(),
            wasm_hash: crypto::sha256_hash(&wasm_bytes),
            client_configs: Some(ClientContractConfigs {
                contract_id,
                env: env.clone(),
                source_account: account.clone(),
            }),
        };

        let retrieved_id = contract_with_configs.contract_id();
        assert!(retrieved_id.is_some());
        assert_eq!(retrieved_id.unwrap().0, contract_id.0);
    }

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
        let simulate_transaction_envelope_result = mock_simulate_tx_response(None);
        let signer_1_account_id = mock_signer1().account_id().0.to_string();
        let get_account_result = mock_account_entry(&signer_1_account_id);

        let env = mock_env(
            Some(Ok(get_account_result)),
            Some(Ok(simulate_transaction_envelope_result)),
            None,
        );
        let wasm_path = "path/to/wasm";
        let mut account = Account::single(mock_signer1());
        let client_configs = ClientContractConfigs {
            contract_id: mock_contract_id(account.clone(), &env),
            env: env.clone(),
            source_account: account.clone(),
        };
        let file_reader = MockFileReader::new(Ok(b"mock wasm bytes".to_vec()));
        let contract =
            Contract::new_with_reader(wasm_path, Some(client_configs), file_reader).unwrap();

        assert!(contract.upload_wasm(&mut account, &env).await.is_ok());
    }

    #[tokio::test]
    async fn test_upload_wasm_contract_code_already_exists() {
        let simulate_transaction_envelope_result = mock_simulate_tx_response(None);

        let signer_1_account_id = mock_signer1().account_id().0.to_string();
        let get_account_result = mock_account_entry(&signer_1_account_id);

        let send_transaction_result = Err(SorobanHelperError::ContractCodeAlreadyExists);

        let env = mock_env(
            Some(Ok(get_account_result)),
            Some(Ok(simulate_transaction_envelope_result)),
            Some(send_transaction_result),
        );
        let wasm_path = "path/to/wasm";
        let mut account = Account::single(mock_signer1());
        let client_configs = ClientContractConfigs {
            contract_id: mock_contract_id(account.clone(), &env),
            env: env.clone(),
            source_account: account.clone(),
        };
        let file_reader = MockFileReader::new(Ok(b"mock wasm bytes".to_vec()));
        let contract =
            Contract::new_with_reader(wasm_path, Some(client_configs), file_reader).unwrap();

        let res = contract.upload_wasm(&mut account, &env).await;
        // result must be Ok, because the contract code already exists.
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_contract_invoke() {
        let simulate_transaction_envelope_result = mock_simulate_tx_response(None);

        let signer_1_account_id = mock_signer1().account_id().0.to_string();
        let get_account_result = mock_account_entry(&signer_1_account_id);
        let send_transaction_result = mock_transaction_response();

        let env = mock_env(
            Some(Ok(get_account_result)),
            Some(Ok(simulate_transaction_envelope_result)),
            Some(Ok(send_transaction_result)),
        );
        let wasm_path = "path/to/wasm";
        let account = Account::single(mock_signer1());
        let client_configs = ClientContractConfigs {
            contract_id: mock_contract_id(account.clone(), &env),
            env: env.clone(),
            source_account: account.clone(),
        };
        let file_reader = MockFileReader::new(Ok(b"mock wasm bytes".to_vec()));
        let mut contract =
            Contract::new_with_reader(wasm_path, Some(client_configs), file_reader).unwrap();

        let res = contract.invoke("function_name", vec![]).await;
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap().response.result_meta,
            mock_transaction_response().response.result_meta
        );
    }

    #[tokio::test]
    async fn test_contract_deploy() {
        let simulate_transaction_envelope_result = mock_simulate_tx_response(None);
        let signer_1_account_id = mock_signer1().account_id().0.to_string();
        let get_account_result = mock_account_entry(&signer_1_account_id);

        // Create a contract ID value for the mock response
        let contract_val = create_contract_id_val();
        let send_transaction_result = Ok(mock_transaction_response_with_return_value(contract_val));

        let env = mock_env(
            Some(Ok(get_account_result)),
            Some(Ok(simulate_transaction_envelope_result)),
            Some(send_transaction_result),
        );
        let wasm_path = "path/to/wasm";
        let mut account = Account::single(mock_signer1());
        let client_configs = ClientContractConfigs {
            contract_id: mock_contract_id(account.clone(), &env),
            env: env.clone(),
            source_account: account.clone(),
        };
        let file_reader = MockFileReader::new(Ok(b"mock wasm bytes".to_vec()));

        let wasm_hash = crypto::sha256_hash(b"mock wasm bytes");
        let contract =
            Contract::new_with_reader(wasm_path, Some(client_configs), file_reader).unwrap();
        let res = contract.deploy(&env, &mut account, None).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap().wasm_hash, wasm_hash);
    }

    #[test]
    fn test_set_client_configs() {
        let wasm_bytes = b"mock wasm bytes".to_vec();
        let mut contract = Contract {
            wasm_bytes: wasm_bytes.clone(),
            wasm_hash: crypto::sha256_hash(&wasm_bytes),
            client_configs: None,
        };

        let env = mock_env(None, None, None);
        let account = Account::single(mock_signer1());
        let contract_id = mock_contract_id(account.clone(), &env);

        let configs = ClientContractConfigs {
            contract_id,
            env: env.clone(),
            source_account: account.clone(),
        };

        contract.set_client_configs(configs.clone());

        assert!(contract.client_configs.is_some());
        let set_configs = contract.client_configs.unwrap();
        assert_eq!(set_configs.contract_id.0, contract_id.0);
    }

    #[test]
    fn test_from_configs() {
        let env = mock_env(None, None, None);
        let account = Account::single(mock_signer1());
        let contract_id = mock_contract_id(account.clone(), &env);

        let client_configs = ClientContractConfigs {
            contract_id,
            env: env.clone(),
            source_account: account.clone(),
        };
        let contract = Contract::from_configs(client_configs.clone());

        assert!(contract.client_configs.is_some());
        let stored_configs = contract.client_configs.unwrap();
        assert_eq!(stored_configs.contract_id.0, contract_id.0);

        // Verify WASM bytes are empty
        assert!(contract.wasm_bytes.is_empty());
        assert_eq!(contract.wasm_hash, crypto::sha256_hash(&[]));
    }
}
