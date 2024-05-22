use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, Result};

pub fn dotenv_macro(input: Ident) -> Result<TokenStream> {
    let path = std::env::current_dir().unwrap().join(".env");
    let s = std::fs::read_to_string(path)
        .map_err(|_| syn::Error::new(Span::call_site(), ".env not found in current dir"))?;

    let pairs = s
        .lines()
        .map(|line| line.split("=").map(|s| s.trim().trim_matches('"')))
        .filter_map(|mut s| match (s.next(), s.next()) {
            (Some(key), Some(value)) => Some((key, value)),
            _ => None,
        })
        .map(|(key, value)| {
            let ident = Ident::new(&key.to_lowercase(), Span::call_site());
            (ident, value)
        })
        .collect::<Vec<_>>();

    let fields = pairs.iter().map(|(ident, _value)| {
        quote! {
            #ident: &'static str
        }
    });

    let instance_fields = pairs.iter().map(|(ident, value)| {
        quote! {
            #ident: #value
        }
    });

    Ok(quote! {
        #[derive(Clone, Debug, PartialEq, Default)]
        struct DotEnv {
            #(#fields,)*
        }

        pub const #input: DotEnv = DotEnv { #(#instance_fields,)* };
    })
}
