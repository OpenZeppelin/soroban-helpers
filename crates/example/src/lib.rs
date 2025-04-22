#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, vec, Address, Env, String, Symbol, Vec};

#[contract]
pub struct Token;

const KEY: Symbol = symbol_short!("value");

#[contractimpl]
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

mod test;
