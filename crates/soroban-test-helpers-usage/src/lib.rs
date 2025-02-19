#![no_std]
use soroban_sdk::{Address, contract, contractimpl, vec, Env, String, Vec};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn send(env: &Env, from: Address, to: Address) -> Vec<String> {
        let from_str = from.to_string();
        let to_str = to.to_string();
        vec![&env, from_str, to_str]
    }
}

mod test;
