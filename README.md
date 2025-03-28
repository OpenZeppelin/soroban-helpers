# Soroban Helpers

[![codecov](https://codecov.io/gh/OpenZeppelin/soroban-helpers/graph/badge.svg?token=YIV3ZHYZKN)](https://codecov.io/gh/OpenZeppelin/soroban-helpers)

A collection of Rust libraries designed to simplify development and testing with Soroban, the smart contract platform for the Stellar network.

## Overview

This project provides three main components:

1. **soroban-rs**: A high-level client library for interacting with the Soroban RPC API
2. **soroban-rs-macros**: A procedural macro that introduces `soroban!` macro for generating smart contract clients from Soroban contract definitions.
3. **soroban-test-helpers**: A procedural macro library that simplifies writing tests for Soroban smart contracts
4. **example**: An example contract demonstrating how to use the test helpers.

## Components

### soroban-rs

A high-level client library that abstracts away the complexity of interacting with the Soroban RPC API. It provides:

- **Env**: Manages connections to Soroban RPC endpoints and handles network configuration
- **Signer**: Manages transaction signing with Stellar keypairs
- **Account**: Manages higher level account interactions.
- **Contract**: Simplifies contract deployment and interaction, including support for constructor arguments
- **Transaction Builder**: Helps create and manage Soroban transactions

#### Examples

For detailed examples of how to use `soroban-rs`, please refer to the [examples](crates/soroban-rs/examples) directory. The examples include:

- `deploy_and_invoke.rs`: Demonstrates deploying and invoking a contract.
- `invoke_contract.rs`: Demonstrates invoking an already deployed contract.
- `create_multisig.rs`: Demonstrates adding signers to an existing account.

### soroban-rs-macros

A procedural macro that introduces `soroban!` macro for generating smart contract clients from Soroban contract definitions.

#### Example

```rust
use soroban_rs_macros::soroban;

soroban!(r#"
    pub struct Token;

    impl Token {
        pub fn transfer(env: &Env, from: Address, to: Address, amount: u128) -> bool {
            // Contract implementation...
        }
    }
"#);

async fn main() {
    let token = TokenClient::new();
    let res =token.transfer(from, to, amount).await;
}
```

### soroban-test-helpers

A procedural macro library that simplifies writing tests for Soroban smart contracts by automatically initializing the test environment and test accounts.

The `#[test]` macro transforms test functions to automatically:

1. Create a default Soroban environment
2. Generate test addresses
3. Inject these as arguments to your test function

#### Example Usage

Instead of manually creating the test environment like this:

```rust
#[test]
fn test() {
    let env = Env::default();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    
    // Test contract functionality
    // ...
}
```

You can use the simplified approach with soroban-test-helpers:

```rust
#[soroban_test_helpers::test]
fn test_injected_args(e: Env, alice: Address, bob: Address) {
    // Test contract functionality directly with injected arguments
    // ...
}
```

### Example

An example crate demonstrating how to use the `soroban-test-helpers`, `soroban-rs-macros` and `soroban-rs` libraries for testing Soroban contracts. This serves as a reference implementation to show how to:

- Write testsusing both traditional methods and the simplified `soroban-test-helpers` approach.
- Add scripts to your soroban project for deploying and invoking contracts.
- Use the `soroban!` macro to generate client code for interacting with Soroban contracts.

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
