use std::path::MAIN_SEPARATOR_STR;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Ident, LitStr, Result};

pub fn static_files_macro(input: DeriveInput) -> Result<TokenStream> {
    let struct_ident = input.ident;
    let Data::Struct(_) = input.data else {
        panic!("Only structs are supported");
    };
    let Some(path) = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("folder"))
        .filter_map(|attr| attr.parse_args::<LitStr>().ok())
        .last()
    else {
        return Ok(quote! {});
    };
    let Some(prefix) = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("prefix"))
        .filter_map(|attr| attr.parse_args::<LitStr>().ok())
        .last()
    else {
        return Ok(quote! {});
    };
    let path = std::env::current_dir().unwrap().join(path.value());
    let p1 = path.clone();
    let root_str = p1.to_string_lossy();
    let files = std::fs::read_dir(path)
        .unwrap()
        .into_iter()
        .filter_map(|dir_entry| dir_entry.ok())
        .filter(|file| match file.file_type() {
            Ok(file_type) => file_type.is_file(),
            Err(_) => false,
        })
        .map(|dir_entry| dir_entry.path())
        .collect::<Vec<_>>();
    let consts = files.iter().map(|path| {
        let ident_name = path
            .clone()
            .with_extension("")
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_uppercase();
        let bytes_ident = Ident::new(&format!("{}_BYTES", &ident_name), Span::call_site());
        let hash_ident = Ident::new(&format!("{}_HASH", &ident_name), Span::call_site());
        let filename = path.file_name().unwrap().to_string_lossy();
        quote! {
            const #bytes_ident: &'static [u8] = include_bytes!(concat!(#root_str, "/", #filename));
            const #hash_ident: u64 = Self::hash(Self::#bytes_ident);
        }
    });
    let pfx_value = &prefix.value();
    let pfx = match pfx_value.as_str() {
        "" => MAIN_SEPARATOR_STR,
        val => val,
    };
    let rendered = files.iter().map(|path| {
        let f1 = path.clone();
        let ident_name = path.clone().with_extension("").file_name().unwrap().to_string_lossy().to_uppercase();
        let hash_ident = Ident::new(&format!("{}_HASH", &ident_name), Span::call_site());
        let filename = f1.file_name().unwrap().to_string_lossy();
        let filename = format!("{}{}", pfx, filename);
        if let Some(ext) = path.extension() {
            if let Some(ext) = ext.to_str() {
                match ext {
                    "js" => quote! {
                        <script src={format!("{}?v={}", #filename, Self::#hash_ident)} defer></script>
                    },
                    "css" => quote! {
                        <link rel="stylesheet" href={format!("{}?v={}", #filename, Self::#hash_ident)} />
                    },
                    _ => quote! {}
                }
            } else {
                quote! {}
            }
        } else {
            quote! {}
        }
    });
    let get_matches = files.iter().map(|path| {
        let ident_name = path
            .clone()
            .with_extension("")
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_uppercase();
        let bytes_ident = Ident::new(&format!("{}_BYTES", &ident_name), Span::call_site());
        let content_type = if let Some(ext) = path.extension() {
            if let Some(ext) = ext.to_str() {
                match ext {
                    "js" => "text/javascript",
                    "css" => "text/css",
                    "wasm" => "application/wasm",
                    _ => "application/octect-stream",
                }
            } else {
                "application/octet-stream"
            }
        } else {
            "application/octet-stream"
        };
        let filename = format!(
            "{}{}",
            &pfx_value,
            path.file_name().unwrap().to_string_lossy()
        );
        quote! {
            #filename => {
                Some((#content_type, Self::#bytes_ident))
            }
        }
    });

    Ok(quote! {
        impl #struct_ident {
            #(#consts)*

            pub fn get<'a, 'b>(uri: &'a str) -> Option<(&'b str, &'static [u8])> {
                match uri {
                    #(#get_matches,)*
                    _ => None
                }
            }

            pub fn render() -> ryde::Component {
                html! {
                    <>
                        #(#rendered)*
                    </>
                }
            }

            pub const fn hash(bytes: &[u8]) -> u64 {
                let mut hash = 0xcbf29ce484222325;
                let prime = 0x00000100000001B3;
                let mut i = 0;

                while i < bytes.len() {
                    hash ^= bytes[i] as u64;
                    hash = hash.wrapping_mul(prime);
                    i += 1;
                }

                hash
            }
        }
    })
}
