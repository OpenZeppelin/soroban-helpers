use dotenv::from_path;
use ed25519_dalek::SigningKey;
use soroban_rs::{
    Account, Contract, Env, EnvConfigs, Signer,
    xdr::{ScAddress, ScVal},
};
use std::{env, path::Path};
use stellar_strkey::ed25519::PrivateKey;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // Path to the contract wasm file
    let contract_path =
        "../../target/wasm32-unknown-unknown/release/soroban_test_helpers_usage.wasm";
    let contract = Contract::new(contract_path, None)?;

    // Deploys the contract
    let mut deployed = contract
        .deploy(&env, &mut account, Some(vec![ScVal::U32(42)]))
        .await?;

    println!(
        "Contract deployed successfully with ID: {:?}",
        deployed.contract_id()
    );

    // Calls send function in contract from Alice and Bob
    let alice = ScVal::Address(ScAddress::Account(account.account_id()));
    let bob = ScVal::Address(ScAddress::Account(account.account_id()));
    let invoke_res = deployed.invoke("send", vec![alice, bob]).await;

    match invoke_res {
        Ok(res) => println!("Contract invoked successfully with result {:?}", res),
        Err(e) => println!("Contract invocation failed as expected: {}", e),
    }
    Ok(())
}
