use dotenv::from_path;
use ed25519_dalek::SigningKey;
use soroban_rs::{Account, AccountConfig, Parser, ParserType, Provider, ProviderConfigs, Signer};
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
    let signer_1 = Signer::new(SigningKey::from_bytes(&private_key_1.0));
    let signer_2 = Signer::new(SigningKey::from_bytes(&private_key_2.0));
    let signer_3 = Signer::new(SigningKey::from_bytes(&private_key_3.0));

    // Create account that will become multisig
    let target_account = Account::account(signer_3.clone());

    // Setup provider
    let provider = Provider::new(ProviderConfigs {
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
    let tx_envelope = target_account.configure(&provider, config).await?;

    // Send transaction
    let response = provider
        .send_transaction(&tx_envelope)
        .await
        .expect("Failed to send transaction");

    let parser = Parser::new(ParserType::AccountSetOptions);
    let result = parser.parse(&response)?;

    println!("{:?}", result);
    Ok(())
}
