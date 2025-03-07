use dotenv::from_path;
use soroban_rs::{Contract, Provider, Signer,  xdr::{ScAddress, ScVal}};
use std::{env, path::Path};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    from_path(Path::new("examples/.env")).ok();

    let private_key =
        env::var("SOROBAN_PRIVATE_KEY").expect("SOROBAN_PRIVATE_KEY must be set in .env file");

    let provider = Provider::new(
        "https://soroban-testnet.stellar.org",
        "Test SDF Network ; September 2015",
    )?;

    let signer = Signer::new(&private_key)?;
    let contract = Contract::new(
        "../../target/wasm32-unknown-unknown/release/soroban_test_helpers_usage.wasm",
    )?;

    // Deploy contract with constructor argument (u32 value of 42)
    let constructor_args = Some(vec![ScVal::U32(42)]);
    let contract_id = contract
        .deploy(&provider, &signer, constructor_args)
        .await?;

    println!("Contract deployed successfully with ID: {:?}", contract_id);

    let alice = ScVal::Address(ScAddress::Account(signer.account_id()));
    let bob = ScVal::Address(ScAddress::Account(signer.account_id()));

    let invoke_res = contract
        .invoke(&contract_id, "send", vec![alice, bob], &provider, &signer)
        .await?;

    println!("Contract invoked successfully with result {:?}", invoke_res);
    Ok(())
}
