use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, FnArg, Item, ReturnType, File, 
};

#[proc_macro]
pub fn soroban(input: TokenStream) -> TokenStream {
    let input_str = parse_macro_input!(input as syn::LitStr).value();
    let file_ast: File = syn::parse_str(&input_str).expect("Failed to parse input");

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
        let transformed_inputs = method.sig.inputs.iter().skip(1).map(|arg| {
            match arg {
                FnArg::Typed(pat_type) => {
                    let pat = &pat_type.pat;
                    quote! { #pat : soroban_rs::xdr::ScVal }
                },
                FnArg::Receiver(r) => quote! { #r },
            }
        });
    
        // Transform return type to ScVal
        let transformed_output = match &method.sig.output {
            ReturnType::Default => quote! {},
            ReturnType::Type(_, _) => quote! { -> soroban_rs::xdr::ScVal },
        };
    
        Some(quote! {
            pub async fn #method_name(&self, #(#transformed_inputs),*) #transformed_output {
                println!("Calling {} on contract {}", stringify!(#method_name), self.contract_id);
                ScVal::Void
            }
        })
    });

    let expanded = quote! {
        pub struct #client_struct_ident {
            env: soroban_rs::Env,
            contract_id: soroban_rs::ContractId,
        }

        impl #client_struct_ident {
            #(#transformed_methods)*
            pub fn new(env: &soroban_rs::Env, contract_id: &soroban_rs::ContractId) -> Self {
                Self {
                    env: env.clone(),
                    contract_id: contract_id.clone(),
                }
            }

        }
    };

    expanded.into()
}