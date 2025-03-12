#![cfg(test)]

use super::*;
use soroban_sdk::{Address, Env, testutils::Address as AddressTrait, vec};

// Default test implementation.
#[test]
fn test() {
    let env = Env::default();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let contract_id = env.register(Token, (19_u32,));
    let client = TokenClient::new(&env, &contract_id);

    let words = client.send(&alice, &bob);
    assert_eq!(words, vec![&env, alice.to_string(), bob.to_string(),]);
}

// Test implementation using injected arguments.
#[soroban_test_helpers::test]
fn test_injected_args(env: Env, alice: Address, bob: Address) {
    let contract_id = env.register(Token, (19_u32,));
    let client = TokenClient::new(&env, &contract_id);

    let words = client.send(&alice, &bob);
    assert_eq!(words, vec![&env, alice.to_string(), bob.to_string(),]);
}
