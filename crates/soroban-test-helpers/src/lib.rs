
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg};

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as syn::ItemFn);
    let attrs = &item_fn.attrs;
    let sig = &item_fn.sig;
    let fn_name = &sig.ident;
    let fn_return_type = &sig.output;
    let fn_block = &item_fn.block;
    let fn_args = &sig.inputs;

    // Whether 1 or none contracts will be declared.
    let arg_binding_and_ty = match fn_args
      .into_iter()
      .map(|arg| {
        let FnArg::Typed(arg) = arg else {
          return Err(syn::Error::new_spanned(arg, "unexpected receiver argument in test signature"));
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

    // extracts the first Env argument and intializes with ::default()
    let first_ty = arg_binding_and_ty.first()
        .map(|(_binding, ty)| ty)
        .expect("at least one argument required");

    let env_init = quote! { let env = <#first_ty>::default(); };

    let arg_inits = arg_binding_and_ty.iter().enumerate().map(|(i, (_arg_binding, arg_ty))| {
        if i == 0 { 
            // Use 'env' instead of 'e'
            quote! { env.clone() }
        } else {
            // Use 'env' instead of 'e'
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