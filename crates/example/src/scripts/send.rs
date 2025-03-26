use dotenv::dotenv;
use ed25519_dalek::SigningKey;
use soroban_rs::{
    Account, ClientContractConfigs, ContractId, Env, EnvConfigs, GetTransactionResponse,
    ParseResult, Parser, ParserType, Signer, SorobanHelperError,
    xdr::{ScAddress, ScVal},
    soroban
};
use std::{env, error::Error};
use stellar_strkey::ed25519::PrivateKey;

soroban!(
    r#"
    pub struct Token;

    impl Token {
        pub fn __constructor(env: Env, value: u32) {
            env.storage().instance().set(&KEY, &value);
        }

        pub fn send(env: &Env, from: Address, to: Address) -> Vec<String> {
            let from_str = from.to_string();
            let to_str = to.to_string();
            vec![&env, from_str, to_str]
        }
    }
"#
);

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
    account.set_authorized_calls(2);

    let contract_id =
        ContractId::from_string("CDJJN63F35UQA5FQTW77FTWO3VFF3PP2KD4AZ3BODTZE2XCDEMGRSWHI")?;

    let client_configs = ClientContractConfigs {
        env: env.clone(),
        contract_id: contract_id.clone(),
        account: account.clone(),
    };
    let mut token_client = TokenClient::new(&client_configs);

    let alice = ScVal::Address(ScAddress::Account(account.account_id()));
    let bob = ScVal::Address(ScAddress::Account(account.account_id()));
    let res: Result<GetTransactionResponse, SorobanHelperError> =
        token_client.send(alice, bob).await;

    let parser = Parser::new(ParserType::InvokeFunction);
    let result = parser.parse(&res.unwrap())?;

    match result {
        ParseResult::InvokeFunction(Some(sc_val)) => {
            println!("Invocation result: {:?}", sc_val);
            Ok(())
        }
        _ => Err("Failed to parse InvokeFunction result".into()),
    }
}
