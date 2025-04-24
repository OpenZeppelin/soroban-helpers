# Soroban Macros

A procedural macro library that simplifies working with Soroban smart contracts by automatically generating client code.

### soroban! Macro

The `soroban!` macro automatically generates client code for interacting with Soroban contracts by:

- Parsing contract interface from Rust code
- Creating type-safe client structs with matching methods
- Handling parameter transformations and RPC communication
- Converting parameters to ScVal types

## Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
soroban-rs-macros = { version = "0.2.3" }
soroban-rs = { version = "0.2.3" }
```

### Example

```rust
use soroban_rs_macros::soroban;
use soroban_rs::{xdr::ScVal, ClientContractConfigs, GetTransactionResponse, SorobanHelperError};

// Define your contract interface
soroban!(r#"
    pub struct Token;

    impl Token {
        pub fn transfer(env: &Env, from: Address, to: Address, amount: u128) -> bool {
            // Contract implementation...
        }
    }
"#);

// Use the generated client
async fn use_token_client() -> Result {
    // Set up client configuration
    let client_configs = ClientContractConfigs {
        // ... configuration details
    };
    
    // Create client instance
    let mut token_client = TokenClient::new(&client_configs);
    
    // Call contract method with ScVal parameters
    let from_scval = /* ... */;
    let to_scval = /* ... */;
    let amount_scval = /* ... */;
    
    token_client.transfer(from_scval, to_scval, amount_scval).await
}
```

## Generated Code

For a contract named `Token`, the macro generates:

- A `TokenClient` struct with client configuration
- A `new` method to instantiate the client passing the deployed contract configs:
    - Contract ID
    - Env (RPC url + passphrase)
    - Account to be used to send the transactions
- Methods matching the contract's public interface.

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

This project is licensed under MIT - see the [LICENSE](LICENSE) file for details.

## Security

If you discover a security vulnerability within this project, please see [SECURITY.md](SECURITY.md) for instructions on responsible disclosure.

## Maintainers

See [CODEOWNERS](CODEOWNERS) file for the list of project maintainers.