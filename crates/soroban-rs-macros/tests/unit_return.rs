//! Regression test: the `soroban!` macro must emit a `Result<SorobanTransactionResponse,
//! SorobanHelperError>` return type for every generated client method, even when the source
//! contract method returns unit (`()`).
//!
//! The generated body is always `self.contract.invoke(...).await`, which yields
//! `Result<SorobanTransactionResponse, SorobanHelperError>`. If the macro emits an empty
//! return type for unit-returning source methods, the compiler rejects the expansion with
//! E0308: `expected (), found Result<...>`.

use soroban_rs_macros::soroban;

soroban!(
    r#"
    pub struct UnitReturn;

    impl UnitReturn {
        pub fn no_return(env: &Env) {
            // Source returns (). Generated client method must still return
            // Result<SorobanTransactionResponse, SorobanHelperError>.
        }

        pub fn no_return_with_args(env: &Env, a: u32, b: u32) {
            // Same, with arguments.
        }

        pub fn with_return(env: &Env) -> u32 {
            // Control case: already worked before the fix.
            0
        }
    }
"#
);

// Compile-only: pin the generated return types. If the macro regresses to emitting no return
// type for unit-returning source methods, this function fails to type-check.
#[allow(dead_code)]
async fn _assert_signatures(client: &mut UnitReturnClient) {
    let _: Result<soroban_rs::SorobanTransactionResponse, soroban_rs::SorobanHelperError> =
        client.no_return().await;

    let a = soroban_rs::xdr::ScVal::U32(1);
    let b = soroban_rs::xdr::ScVal::U32(2);
    let _: Result<soroban_rs::SorobanTransactionResponse, soroban_rs::SorobanHelperError> =
        client.no_return_with_args(a, b).await;

    let _: Result<soroban_rs::SorobanTransactionResponse, soroban_rs::SorobanHelperError> =
        client.with_return().await;
}

#[test]
fn macro_expansion_compiles_for_unit_return_methods() {
    // The real assertion is that this file compiles. Having a live test case keeps
    // `cargo test -p soroban-rs-macros --test unit_return` meaningful.
}
