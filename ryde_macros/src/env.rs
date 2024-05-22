use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, Result};

pub fn dotenv_macro() -> Result<TokenStream> {
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
            #ident: String
        }
    });

    let instance_fields = pairs.iter().map(|(ident, value)| {
        quote! {
            #ident: #value.into()
        }
    });

    Ok(quote! {
        #[derive(Clone, Debug, PartialEq)]
        struct Env {
            #(#fields,)*
        }

        pub fn dotenv() -> Env {
            Env {
                #(#instance_fields,)*
            }
        }
    })
}
