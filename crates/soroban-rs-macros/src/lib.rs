//! # Soroban Macros
//!
//! This crate provides procedural macros for working with Soroban smart contracts.
//!
//! ## Features
//!
//! - `soroban!` macro: Automatically generates client code for interacting with Soroban contracts by:
//!   - Parsing contract interface from Rust code
//!   - Creating type-safe client structs with matching methods
//!   - Handling parameter transformations and RPC communication
//!
//! ## Example
//!
//! ```rust,no_run
//! use soroban_rs_macros::soroban;
//! use soroban_rs::{xdr::ScVal, ClientContractConfigs};
//!
//! soroban!(r#"
//!     pub struct Token;
//!
//!     impl Token {
//!         pub fn transfer(env: &Env, from: Address, to: Address, amount: u128) -> bool {
//!             // Contract implementation...
//!         }
//!     }
//! "#);
//!
//! // Generated client can be used like this:
//! async fn use_token_client() {
//!     let client_configs = ClientContractConfigs::new(/* ... */);
//!     let mut token_client = TokenClient::new(&client_configs);
//!     
//!     // Call the contract method with ScVal parameters
//!     let result = token_client.transfer(from_scval, to_scval, amount_scval).await;
//! }
//! ```
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, File, FnArg, Item, ReturnType};

/// A procedural macro for generating Soroban contract client code.
///
/// This macro parses a Soroban contract interface and generates a client struct with
/// corresponding methods that can be used to interact with the deployed contract.
///
/// # How It Works
///
/// 1. Parses the provided Rust code containing a contract struct and implementation
/// 2. Extracts the contract's public methods
/// 3. Generates a client struct with matching methods that:
///    - Skip the first parameter (env)
///    - Convert all other parameters to use `ScVal` types
///    - Return `Result<GetTransactionResponse, SorobanHelperError>`
///
/// # Parameters
///
/// * `input`: A string literal containing the Rust code of the contract interface
///
/// # Generated Code
///
/// For a contract named `Token`, the macro generates:
/// - A `TokenClient` struct with client configuration
/// - Methods matching the contract's public interface but with modified signatures
/// - A `new` method to instantiate the client
///
/// # Example
///
/// ```rust,no_run
/// soroban!(r#"
///     pub struct Counter;
///
///     impl Counter {
///         pub fn increment(env: &Env, amount: u32) -> u32 {
///             // Contract implementation...
///         }
///     }
/// "#);
///
/// // Use the generated client:
/// let mut counter_client = CounterClient::new(&client_configs);
/// let result = counter_client.increment(amount_scval).await?;
/// ```
#[proc_macro]
pub fn soroban(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as syn::LitStr);
    let value = lit.value();

    let code = if value.ends_with(".rs") {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let full_path = std::path::Path::new(&manifest_dir).join(&value);
        std::fs::read_to_string(&full_path)
            .unwrap_or_else(|_| panic!("Failed to read file at {}", full_path.display()))
    } else {
        value
    };
    let file_ast: File = syn::parse_str(&code).expect("Failed to parse input");

    let mut struct_name = None;
    let mut methods = Vec::new();

    // Find struct and impl methods
    for item in file_ast.items {
        match item {
            Item::Struct(item_struct) => struct_name = Some(item_struct.ident),
            Item::Impl(impl_block) => {
                for impl_item in impl_block.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        methods.push(method);
                    }
                }
            }
            _ => (),
        }
    }

    let struct_ident = struct_name.expect("No struct found");
    let client_struct_ident = format_ident!("{}Client", struct_ident);

    // Transform each method according to your requirement
    let transformed_methods = methods.iter().filter_map(|method| {
        let method_name = &method.sig.ident;
        let method_name_str = method_name.to_string();

        // Skip __constructor or new() methods
        if method_name_str == "__constructor" || method_name_str == "new" {
            return None;
        }

        // Transform inputs: Skip first arg (env), transform rest
        let transformed_inputs: Vec<_> = method
            .sig
            .inputs
            .iter()
            .skip(1)
            .map(|arg| match arg {
                FnArg::Typed(pat_type) => {
                    let pat = &pat_type.pat;
                    quote! { #pat : soroban_rs::xdr::ScVal }
                }
                FnArg::Receiver(r) => quote! { #r },
            })
            .collect();

        // Also create a list of just the parameter names for the invoke call
        let param_names = method
            .sig
            .inputs
            .iter()
            .skip(1)
            .map(|arg| match arg {
                FnArg::Typed(pat_type) => {
                    let pat = &pat_type.pat;
                    quote! { #pat }
                }
                FnArg::Receiver(r) => quote! { #r },
            })
            .collect::<Vec<_>>();

        // Transform return type to ScVal
        let transformed_output = match &method.sig.output {
            ReturnType::Default => quote! {},
            ReturnType::Type(_, _) => {
                quote! { -> Result<soroban_rs::GetTransactionResponse, soroban_rs::SorobanHelperError>  }
            }
        };

        Some(quote! {
            pub async fn #method_name(&mut self, #(#transformed_inputs),*) #transformed_output {
                // internally calls invoke API.
                self.contract.invoke(stringify!(#method_name), vec![#(#param_names),*]).await
            }
        })
    });

    let expanded = quote! {
        pub struct #client_struct_ident {
            client_configs: soroban_rs::ClientContractConfigs,
            contract: soroban_rs::Contract,
        }

        impl #client_struct_ident {
            #(#transformed_methods)*
            pub fn new(client_configs: &soroban_rs::ClientContractConfigs) -> Self {
                let contract = soroban_rs::Contract::from_configs(client_configs.clone());
                Self { client_configs: client_configs.clone(), contract }
            }

        }
    };

    expanded.into()
}
