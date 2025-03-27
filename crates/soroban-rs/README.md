# soroban-rs

`soroban-rs` is a Rust library designed to interact with the Soroban smart contract platform on the Stellar network. It provides tools for managing accounts, signing transactions, deploying and invoking smart contracts, and handling cryptographic operations.

## Features

- **Provider**: Connects to the Soroban network using RPC and manages network configurations.
- **Signer**: Handles transaction signing using Ed25519 keys.
- **TransactionBuilder**: Constructs and simulates transactions.
- **Contract**: Manages smart contract deployment and invocation.
- **AccountManager**: Retrieves account details and manages transaction sequences.
- **Crypto Utilities**: Provides cryptographic functions like hashing and salt generation.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
soroban-rs = "0.1.0"
```

## Usage

### Example: Deploying and Invoking a Contract

Here's a basic example of how to deploy and invoke a contract using `soroban-rs`:

```rust
use soroban_rs::{
    Contract, Env, EnvConfigs, Signer,
    xdr::{ScAddress, ScVal},
};
use std::{env, path::Path};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let private_key_str =
        env::var("SOROBAN_PRIVATE_KEY").expect("SOROBAN_PRIVATE_KEY must be set");
    let private_key = PrivateKey::from_string(&private_key_str).expect("Invalid private key");
    let signing_key = SigningKey::from_bytes(&private_key.0);

    let configs = EnvConfigs {
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
    };
    let env = Env::new(configs)?;

    let signer = Signer::new(signing_key)?;
    let mut account = Account::single(Signer::new(signing_key));

    let contract = Contract::new(
        "path/to/contract.wasm",
        None
    )?;

    // Deploy contract with constructor argument (u32 value of 42)
    let constructor_args = Some(vec![ScVal::U32(42)]);
    let mut deployed = contract
        .deploy(&env, &mut account, constructor_args)
        .await?;

    println!("Contract deployed successfully with ID: {:?}", deployed.contract_id());

    let alice = ScVal::Address(ScAddress::Account(signer.account_id()));
    let bob = ScVal::Address(ScAddress::Account(signer.account_id()));

    let invoke_res = deployed
        .invoke("send", vec![alice, bob])
        .await?;

    println!("Contract invoked successfully with result {:?}", invoke_res);
    Ok(())
}
```

## Error Handling

The library uses a custom error type `SorobanHelperError` to handle various errors such as transaction failures, network request failures, and XDR encoding issues.

## Contributing

We welcome contributions from the community! Here's how you can get involved:

1. [Fork the repository](https://github.com/OpenZeppelin/soroban-helpers/fork)
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

If you are looking for a good place to start, find a good first issue [here](https://github.com/OpenZeppelin/soroban-helpers/issues?q=is%3Aissue%20is%3Aopen%20label%3Agood-first-issue).

You can open an issue for a [bug report](https://github.com/OpenZeppelin/soroban-helpers/issues/new?assignees=&labels=T-bug%2CS-needs-triage&projects=&template=bug.yml), [feature request](https://github.com/OpenZeppelin/soroban-helpers/issues/new?assignees=&labels=T-feature%2CS-needs-triage&projects=&template=feature.yml), or [documentation request](https://github.com/OpenZeppelin/soroban-helpers/issues/new?assignees=&labels=T-documentation%2CS-needs-triage&projects=&template=docs.yml).

You can find more details in our [Contributing](CONTRIBUTING.md) guide.

Please read our [Code of Conduct](CODE_OF_CONDUCT.md) and check the [Security Policy](SECURITY.md) for reporting vulnerabilities.

## License

This project is licensed under the GNU Affero General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Security

If you discover a security vulnerability within this project, please see [SECURITY.md](SECURITY.md) for instructions on responsible disclosure.

## Maintainers

See [CODEOWNERS](CODEOWNERS) file for the list of project maintainers.
