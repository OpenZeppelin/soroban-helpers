use soroban_rs::{Provider, Signer, Contract, ScVal};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::new(
        "https://soroban-testnet.stellar.org",
        "Test SDF Network ; September 2015"
    )?;
    
    let mut signer = Signer::new("SDP2YO7KQSHI6K6CUXLZ23EXKLCTLNCSDQOHOUBCNXYRLZTTDNDREU7Z")?;
    let contract = Contract::new("../../target/wasm32-unknown-unknown/release/soroban_test_helpers_usage.wasm")?;
    
    // Deploy contract with constructor argument (u32 value of 42)
    let constructor_args = Some(vec![ScVal::U32(42)]);
    let contract_id = contract.deploy(&provider, &mut signer, constructor_args).await?;

    println!("Contract deployed successfully with ID: {:?}", contract_id);
    Ok(())
}