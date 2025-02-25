# Soroban Helpers

A collection of Rust libraries to simplify development and testing with Soroban, the smart contract platform for the Stellar network.

## Components

### soroban-rs

A high-level client library for interacting with the Soroban RPC API. It provides:

- `Provider`: Manages connections to Soroban RPC endpoints and handles network configuration
- `Signer`: Manages transaction signing with Stellar keypairs
- `Contract`: Simplifies contract deployment and interaction

Example usage:

```rust
use soroban_rs::{Provider, Signer, Contract, ScVal};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Soroban testnet
    let provider = Provider::new(
        "https://soroban-testnet.stellar.org",
        "Test SDF Network ; September 2015"
    )?;
    
    // Initialize signer with secret key
    let mut signer = Signer::new("YOUR_SECRET_KEY")?;
    
    // Load contract WASM
    let contract = Contract::new("path/to/your/contract.wasm")?;
    
    // Deploy contract with constructor argument
    let constructor_args = Some(vec![ScVal::U32(42)]);
    let contract_id = contract.deploy(&provider, &mut signer, constructor_args).await?;

    println!("Contract deployed successfully with ID: {:?}", contract_id);
    Ok(())
}
```

### soroban-test-helpers

A procedural macro library that simplifies writing tests for Soroban smart contracts. It provides:

- `#[test]` macro: Automatically initializes the Soroban environment and test accounts

The macro transforms test functions to automatically:
1. Create a default Soroban environment
2. Generate test addresses
3. Inject these as arguments to your test function

### soroban-test-helpers-usage

An example contract demonstrating how to use the `soroban-test-helpers` library for testing.

Compare the standard test approach:

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

With the simplified approach using `soroban-test-helpers`:

```rust
#[soroban_test_helpers::test]
fn test_injected_args(e: Env, alice: Address, bob: Address) {
    // Test contract functionality directly with injected arguments
    // ...
}
```
