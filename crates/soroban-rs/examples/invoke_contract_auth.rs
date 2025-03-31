use dotenv::from_path;
use ed25519_dalek::SigningKey;
use soroban_rs::{
    Account, ClientContractConfigs, Contract, Env, EnvConfigs, IntoScVal, ParseResult, Parser,
    ParserType, Signer, xdr::ScVal,
};
use std::{env, error::Error, path::Path};
use stellar_strkey::{Contract as ContractId, ed25519::PrivateKey};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    from_path(Path::new("examples/.env")).ok();

    let private_key_1 = PrivateKey::from_string(
        &env::var("SOROBAN_PRIVATE_KEY_1").expect("SOROBAN_PRIVATE_KEY_1 must be set in .env file"),
    )
    .expect("Invalid private key");
    // let private_key_2 = PrivateKey::from_string(
    //     &env::var("SOROBAN_PRIVATE_KEY_2").expect("SOROBAN_PRIVATE_KEY_2 must be set in .env file"),
    // )
    // .expect("Invalid private key");

    // Converts the private key to a signing key
    let signer_1 = Signer::new(SigningKey::from_bytes(&private_key_1.0));
    // let signer_2 = Signer::new(SigningKey::from_bytes(&private_key_2.0));

    // Creates a new environment
    let configs = EnvConfigs {
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
    };
    let env = Env::new(configs)?;

    // Initializes a new account
    let mut account = Account::single(signer_1);
    // let account2 = Account::single(signer_2);

    account.set_authorized_calls(1);

    // Get the contract ID from env (this would be obtained from the deploy step)
    let contract_id =
        ContractId::from_string("CDIOS5F6BPHLLRG3IIBW3TX47NNLIVVM4UHUOPHOVNBSTPTYAKQVVFOG")?;

    // Initialize contract with existing contract ID
    let client_configs = ClientContractConfigs {
        contract_id,
        env: env.clone(),
        account: account.clone(),
    };

    // Path to the contract wasm file (needed for function schemas)
    let mut contract = Contract::new(
        "./fixtures/soroban_auth_contract.wasm",
        Some(client_configs),
    )?;

    println!("Account: {:?}", account.account_id());

    let address_val = account.account_id().try_into_val()?;
    let val = ScVal::U32(10);
    let invoke_res = contract
        .invoke("increment", vec![address_val, val], true)
        .await?;

    let parser = Parser::new(ParserType::InvokeFunction);
    let result = parser.parse(&invoke_res.response)?;

    match result {
        ParseResult::InvokeFunction(Some(sc_val)) => {
            println!("Invocation result: {:?}", sc_val);
            Ok(())
        }
        _ => Err("Failed to parse InvokeFunction result".into()),
    }
}
