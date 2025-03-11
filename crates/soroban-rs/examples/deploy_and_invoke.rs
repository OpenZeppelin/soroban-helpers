use dotenv::from_path;
use ed25519_dalek::SigningKey;
use soroban_rs::{xdr::{ScAddress, ScVal}, Account, Contract, Provider, ProviderConfigs, Signer, SingleAccount};
use stellar_strkey::ed25519::PrivateKey;
use std::{env, path::Path};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    from_path(Path::new("examples/.env")).ok();

    let private_key_str =
        env::var("SOROBAN_PRIVATE_KEY_1").expect("SOROBAN_PRIVATE_KEY must be set in .env file");
    let private_key = PrivateKey::from_string(&private_key_str)
            .expect("Invalid private key");
    let signing_key = SigningKey::from_bytes(&private_key.0); 

    let configs = ProviderConfigs {
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
    };
    let provider = Provider::new(configs)?;

    let signer = Signer::new(signing_key);
    let account = Account::KeyPair(SingleAccount {
        account_id: signer.account_id(),
        signers: vec![signer],
    });
    let contract = Contract::new(
        "../../target/wasm32-unknown-unknown/release/soroban_test_helpers_usage.wasm",
    )?;

    // Deploy contract with constructor argument (u32 value of 42)
    let constructor_args = Some(vec![ScVal::U32(42)]);
    let contract_id = contract
        .deploy(&provider, &account, constructor_args)
        .await?;

    println!("Contract deployed successfully with ID: {:?}", contract_id);

    let alice = ScVal::Address(ScAddress::Account(account.account_id()));
    let bob = ScVal::Address(ScAddress::Account(account.account_id()));

    let invoke_res = contract
        .invoke(&contract_id, "send", vec![alice, bob], &provider, &account)
        .await?;

    println!("Contract invoked successfully with result {:?}", invoke_res);
    Ok(())
}
