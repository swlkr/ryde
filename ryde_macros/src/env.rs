use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, Ident, LitStr, Result, Token};

pub fn env_vars_macro(input: Punctuated<Ident, Token![,]>) -> Result<TokenStream> {
    let fns = input
        .iter()
        .map(|ident| {
            let lit_str = LitStr::new(&ident.to_string(), ident.span());

            quote! {
                #[allow(non_snake_case)]
                fn #ident() -> &'static str {
                    static ENV_VAR: std::sync::OnceLock<String> = std::sync::OnceLock::new();
                    ENV_VAR.get_or_init(|| std::env::var(#lit_str).expect(#lit_str))
                }
            }
        })
        .collect::<Vec<_>>();
    let calls = input
        .iter()
        .map(|ident| {
            quote! {
                let _ = #ident();
            }
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        pub fn env_vars() {
            #(#calls)*
        }

        #(#fns)*
    })
}
