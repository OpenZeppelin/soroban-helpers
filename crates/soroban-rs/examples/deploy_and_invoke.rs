use dotenv::from_path;
use ed25519_dalek::SigningKey;
use soroban_rs::{Account, ClientContractConfigs, Contract, Env, EnvConfigs, IntoScVal, Signer};
use soroban_rs_macros::soroban;
use std::{env, error::Error, path::Path};
use stellar_strkey::ed25519::PrivateKey;

// generates TokenMockClient binding TokenMock contract.
soroban!("fixtures/lib.rs");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    from_path(Path::new("examples/.env")).ok();

    // Loads the private key from the .env file
    let private_key_str =
        env::var("SOROBAN_PRIVATE_KEY_1").expect("SOROBAN_PRIVATE_KEY must be set in .env file");
    let private_key = PrivateKey::from_string(&private_key_str).expect("Invalid private key");

    // Converts the private key to a signing key
    let signing_key = SigningKey::from_bytes(&private_key.0);

    // Creates a new environment
    let configs = EnvConfigs {
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
    };
    let env = Env::new(configs)?;

    // Initializes a new account
    let mut account = Account::single(Signer::new(signing_key));

    // Sets the authorized calls for the account
    // deployment consumes 2 calls (1 for upload wasm, 1 for create)
    account.set_authorized_calls(3);

    println!(
        "Deploying contract using account: {:?}",
        account.account_id().to_string()
    );

    // Path to the contract wasm file
    let contract = Contract::new("./fixtures/soroban-helpers-example.wasm", None)?;

    // Deploys the contract
    let deployed = contract
        .deploy(&env, &mut account, Some(vec![(42 as u32).into_val()]))
        .await?;

    println!(
        "Contract deployed successfully with ID: {:?}",
        deployed
            .contract_id()
            .expect("Contract ID not found")
            .to_string()
    );

    let client_configs = ClientContractConfigs {
        contract_id: deployed.contract_id().expect("Contract ID not found"),
        env: env.clone(),
        account: account.clone(),
    };
    let mut deployed_contract_client = TokenMockClient::new(&client_configs);

    // Calls send function in contract from Alice and Bob
    let alice = account.account_id().try_into_val()?;
    let bob = account.account_id().try_into_val()?;

    let invoke_res = deployed_contract_client.send(alice, bob).await?;

    println!("Result value: {:?}", invoke_res.get_return_value());
    Ok(())
}
