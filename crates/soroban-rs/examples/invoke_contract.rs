use dotenv::from_path;
use ed25519_dalek::SigningKey;
use soroban_rs::{
    Account, ClientContractConfigs, Contract, Env, EnvConfigs, ParseResult, Parser, ParserType,
    Signer,
    xdr::{ScAddress, ScVal},
};
use std::{env, path::Path};
use stellar_strkey::{Contract as ContractId, ed25519::PrivateKey};

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
    let env = Env::new(EnvConfigs {
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
    })?;

    // Initializes a new account
    let mut account = Account::single(Signer::new(signing_key));

    // Sets the authorized calls for the account
    account.set_authorized_calls(1_u16);

    // Get the contract ID from env (this would be obtained from the deploy step)
    let contract_id =
        ContractId::from_string("CARNMCLJQ5OCV7AG7XACKLRBQSFLY7GGZTYVCYULSPRJXWQ37UZUNBCF")?;

    // Initialize contract with existing contract ID
    let client_configs = ClientContractConfigs {
        contract_id,
        env: env.clone(),
        account: account.clone(),
    };

    // Path to the contract wasm file (needed for function schemas)
    let mut contract = Contract::new(
        "./fixtures/soroban-helpers-example.wasm",
        Some(client_configs),
    )?;

    // Calls send function in contract from Alice and Bob
    let alice = ScVal::Address(ScAddress::Account(account.account_id()));
    let bob = ScVal::Address(ScAddress::Account(account.account_id()));

    let invoke_res = contract.invoke("send", vec![alice, bob]).await?;

    let parser = Parser::new(ParserType::InvokeFunction);
    let result = parser.parse(&invoke_res)?;

    match result {
        ParseResult::InvokeFunction(Some(sc_val)) => {
            println!("Invocation result: {:?}", sc_val);
            Ok(())
        }
        _ => Err("Failed to parse InvokeFunction result".into()),
    }
}
