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
        let transformed_inputs: Vec<_> = method.sig.inputs.iter().skip(1).map(|arg| {
            match arg {
                FnArg::Typed(pat_type) => {
                    let pat = &pat_type.pat;
                    quote! { #pat : soroban_rs::xdr::ScVal }
                },
                FnArg::Receiver(r) => quote! { #r },
            }
        }).collect();

        // Also create a list of just the parameter names for the invoke call
        let param_names = method.sig.inputs.iter().skip(1).map(|arg| {
            match arg {
                FnArg::Typed(pat_type) => {
                    let pat = &pat_type.pat;
                    quote! { #pat }
                },
                FnArg::Receiver(r) => quote! { #r },
            }
        }).collect::<Vec<_>>();

        // Transform return type to ScVal
        let transformed_output = match &method.sig.output {
            ReturnType::Default => quote! {},
            ReturnType::Type(_, _) => quote! { -> Result<GetTransactionResponse, soroban_rs::SorobanHelperError>  },
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