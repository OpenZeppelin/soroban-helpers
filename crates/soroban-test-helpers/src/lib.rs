//! # Soroban Test Helpers
//!
//! This crate provides helpful macros and utilities for testing Soroban smart contracts.
//!
//! ## Features
//!
//! - `#[test]` attribute macro: Simplifies writing tests for Soroban contracts by:
//!   - Automatically creating a test environment
//!   - Generating test addresses as needed
//!   - Reducing boilerplate in test code
//!
//! ## Example
//!
//! ```rust,ignore
//! use soroban_test_helpers::test;
//! use soroban_sdk::{Env, Address};
//!
//! #[test]
//! fn my_soroban_contract_test(env: Env, user: Address) {
//!     // Test logic here
//!     // `env` is automatically created with default settings
//!     // `user` is automatically generated with env
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg};

/// A procedural macro for simplifying Soroban contract tests.
///
/// This macro transforms a function into a proper Soroban test by:
///
/// 1. Creating a test environment automatically
/// 2. Generating address arguments automatically
/// 3. Wrapping the test in a proper `#[test]` attribute
///
/// # Parameters
///
/// * The first parameter must be an environment type (`Env`) which will be instantiated using `Default::default()`
/// * Any additional parameters will be auto-generated based on their type:
///   - For `Address` types: generated using `Address::generate(&env)`
///   - For other Soroban data types: must support a similar `generate(&env)` pattern
///   - All generated values are properly passed to your test function
///
/// # Example
///
/// ```rust,no_run
/// #[test]
/// fn transfer_test(env: Env, sender: Address, receiver: Address) {
///     // Test logic here
///     // env will be created with Env::default()
///     // sender and receiver will be created with Address::generate(&env)
/// }
/// ```
#[proc_macro_attribute]
pub fn test(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as syn::ItemFn);
    let attrs = &item_fn.attrs;
    let sig = &item_fn.sig;
    let fn_name = &sig.ident;
    let fn_return_type = &sig.output;
    let fn_block = &item_fn.block;
    let fn_args = &sig.inputs;

    let arg_binding_and_ty = match fn_args
        .into_iter()
        .map(|arg| {
            let FnArg::Typed(arg) = arg else {
                return Err(syn::Error::new_spanned(
                    arg,
                    "unexpected receiver argument in test signature",
                ));
            };
            let arg_binding = &arg.pat;
            let arg_ty = &arg.ty;
            Ok((arg_binding, arg_ty))
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(res) => res,
        Err(err) => return err.to_compile_error().into(),
    };

    let arg_defs = arg_binding_and_ty.iter().map(|(arg_binding, arg_ty)| {
        quote! {
          #arg_binding: #arg_ty
        }
    });

    // extracts the first Env argument and initializes with ::default()
    let first_ty = arg_binding_and_ty
        .first()
        .map(|(_binding, ty)| ty)
        .expect("at least one argument required");
    let env_init = quote! { let env = <#first_ty>::default(); };

    // extracts the following arguments (Addresses) and generates them passing the env as parameter.
    let arg_inits = arg_binding_and_ty
        .iter()
        .enumerate()
        .map(|(i, (_arg_binding, arg_ty))| {
            if i == 0 {
                quote! { env.clone() }
            } else {
                quote! { <#arg_ty>::generate(&env) }
            }
        });

    quote! {
        #( #attrs )*
        #[test]
        fn #fn_name() #fn_return_type {
            #env_init
            let test = | #( #arg_defs ),* | #fn_block;
            test( #( #arg_inits ),* )
        }
    }
    .into()
}
