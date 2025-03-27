use dotenv::from_path;
use soroban_rs::{Account, AccountConfig, Env, EnvConfigs, Signer};
use std::{env, path::Path};
use stellar_strkey::ed25519::PrivateKey;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    from_path(Path::new("examples/.env")).ok();

    // Load private keys from environment
    let private_key_1 = PrivateKey::from_string(
        &env::var("SOROBAN_PRIVATE_KEY_1").expect("SOROBAN_PRIVATE_KEY_1 must be set in .env file"),
    )
    .expect("Invalid private key");
    let private_key_2 = PrivateKey::from_string(
        &env::var("SOROBAN_PRIVATE_KEY_2").expect("SOROBAN_PRIVATE_KEY_2 must be set in .env file"),
    )
    .expect("Invalid private key");
    let private_key_3 = PrivateKey::from_string(
        &env::var("SOROBAN_PRIVATE_KEY_3").expect("SOROBAN_PRIVATE_KEY_3 must be set in .env file"),
    )
    .expect("Invalid private key");

    // Create signers
    let signer_1 = Signer::from(&private_key_1.0);
    let signer_2 = Signer::from(&private_key_2.0);
    let signer_3 = Signer::from(&private_key_3.0);

    // Create account that will become multisig
    let target_account = Account::single(signer_3.clone());

    // Setup provider
    let env = Env::new(EnvConfigs {
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
    })?;

    // Create 1-of-3 multisig configuration
    let config = AccountConfig::new()
        .with_master_weight(1)
        .with_thresholds(1, 1, 1)
        .add_signer(signer_1.public_key(), 1)
        .add_signer(signer_2.public_key(), 1);

    // Apply configuration
    let tx_envelope = target_account.configure(&env, config).await?;

    // Send transaction
    let result = env
        .send_transaction(&tx_envelope)
        .await
        .expect("Failed to send transaction");

    println!("Result: {:?}", result.response.status);
    Ok(())
}
