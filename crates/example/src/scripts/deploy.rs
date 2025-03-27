use dotenv::dotenv;
use ed25519_dalek::SigningKey;
use soroban_rs::{Account, Contract, Env, EnvConfigs, Signer, xdr::ScVal};
use std::{env, error::Error};
use stellar_strkey::ed25519::PrivateKey;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Loads the private key from the .env file
    let private_key_str = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set in .env file");
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
    account.set_authorized_calls(2_u16);

    // Path to the contract wasm file
    let contract = Contract::new(
        "../../target/wasm32-unknown-unknown/release/soroban_test_helpers_usage.wasm",
        None,
    )?;

    // Deploys the contract
    let deployed = contract
        .deploy(&env, &mut account, Some(vec![ScVal::U32(42)]))
        .await?;

    println!(
        "Contract deployed successfully with ID: {:?}",
        deployed.contract_id()
    );

    Ok(())
}
