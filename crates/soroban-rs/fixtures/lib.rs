#![no_std]
use soroban_sdk::{Address, Env, String, Symbol, Vec, contract, contractimpl, symbol_short, vec};

#[contract]
pub struct TokenMock;

const KEY: Symbol = symbol_short!("value");

#[contractimpl]
impl TokenMock {
    pub fn __constructor(env: Env, value: u32) {
        env.storage().instance().set(&KEY, &value);
    }

    pub fn send(env: &Env, from: Address, to: Address) -> Vec<String> {
        let from_str = from.to_string();
        let to_str = to.to_string();
        vec![&env, from_str, to_str]
    }
}

