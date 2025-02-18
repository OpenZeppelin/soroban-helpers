use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn, parse::Parse, parse::ParseStream, Token, Ident, LitInt};
use quote::quote;

struct TestArgs {
  accounts: usize,
}

impl Parse for TestArgs {
  fn parse(input: ParseStream) -> syn::Result<Self> {
      let name: Ident = input.parse()?;
      if name != "accounts" {
          return Err(syn::Error::new(name.span(), "expected `accounts`"));
      }
      input.parse::<Token![:]>()?;
      let accounts: LitInt = input.parse()?;
      
      Ok(TestArgs {
          accounts: accounts.base10_parse()?
      })
  }
}

#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as TestArgs);
    let num_addresses = args.accounts;
    
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_block = &input_fn.block;
    
    let address_params = (0..num_addresses).map(|i| {
        let param_name = format!("_address_{}", i);
        let param_ident = syn::Ident::new(&param_name, proc_macro2::Span::call_site());
        quote! { #param_ident: Address }
    });
    
    // Generate the modified function
    let expanded = quote! {
        fn #fn_name(
            _env: Env,
            #(#address_params,)*
        ) {
            #fn_block
        }
    };
    
    expanded.into()
}
