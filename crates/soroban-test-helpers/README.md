# Soroban Test Helpers

A collection of utilities and macros to simplify testing [Soroban](https://soroban.stellar.org) smart contracts.

[![Crates.io](https://img.shields.io/crates/v/soroban-test-helpers)](https://crates.io/crates/soroban-test-helpers)
[![Docs.rs](https://docs.rs/soroban-test-helpers/badge.svg)](https://docs.rs/soroban-test-helpers)
[![License](https://img.shields.io/crates/l/soroban-test-helpers)](https://github.com/stellar/soroban-helpers/blob/main/LICENSE)

## Features

This crate provides helper utilities for writing cleaner, more concise tests for Soroban smart contracts.

- `#[test]` attribute macro - Simplifies test setup by:
  - Automatically initializing the Soroban environment
  - Generating test addresses as needed
  - Reducing test boilerplate

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
soroban-test-helpers = "0.1.0"
```

## Usage Example

```rust
use soroban_sdk::{Address, Env};
use soroban_test_helpers::test;

// Original test setup
#[test]
fn original_test_approach() {
    let env = Env::default();
    let user = Address::generate(&env);
    let contract = Address::generate(&env);
    
    // Test logic...
}

// Simplified test using soroban-test-helpers
#[test]
fn simplified_test(env: Env, user: Address, contract: Address) {
    // Test logic...
    // Environment setup is handled automatically!
}
```

## How It Works

The `#[test]` attribute macro transforms your test function by:

1. Creating an environment using `Default::default()` for the first parameter
2. Generating subsequent address parameters using `Address::generate(&env)`
3. Ensuring your test code runs with these automatically created values

This significantly reduces the amount of boilerplate code in your tests.

## Contributing

We welcome contributions from the community! Here's how you can get involved:

1. Fork the repository
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
