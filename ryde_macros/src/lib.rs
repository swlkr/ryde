mod db;
mod env;
mod html;
mod request_parts;
mod routes;
mod static_files;

use db::{db_macro, SqlExpr};
use env::dotenv_macro;
use html::{component_macro, html_macro};
use proc_macro::TokenStream;
use quote::ToTokens;
use request_parts::derive_request_parts_macro;
use routes::{router_macro, routes_macro, url_macro, StateRouter, Url};
use static_files::static_files_macro;
use syn::{parse_macro_input, punctuated::Punctuated, DeriveInput, Ident, ItemFn, Token};

#[proc_macro]
pub fn db(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input with Punctuated::<SqlExpr, Token![,]>::parse_terminated);
    match db_macro(input) {
        Ok(s) => s.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn routes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as StateRouter);
    match routes_macro(input) {
        Ok(s) => s.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn router(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    match router_macro(input) {
        Ok(s) => s.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn url(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Url);
    match url_macro(input) {
        Ok(s) => s.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(StaticFiles, attributes(folder, prefix))]
pub fn static_files(s: TokenStream) -> TokenStream {
    let input = parse_macro_input!(s as DeriveInput);
    match static_files_macro(input) {
        Ok(s) => s.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    match html_macro(input) {
        Ok(s) => s.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Ident);
    match component_macro(input) {
        Ok(s) => s.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn dotenv(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Ident);
    match dotenv_macro(input) {
        Ok(s) => s.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(RequestParts)]
pub fn derive_request_parts(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_request_parts_macro(input) {
        Ok(s) => s.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}
