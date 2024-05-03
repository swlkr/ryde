mod db;
mod html;
mod routes;
mod static_files;

use db::db_macro;
use html::html_macro;
use proc_macro::TokenStream;
use quote::ToTokens;
use routes::{routes_macro, url_macro, StateRouter};
use static_files::static_files_macro;
use syn::{parse_macro_input, punctuated::Punctuated, DeriveInput, Expr, ExprAssign, Token};

#[proc_macro]
pub fn db(input: TokenStream) -> TokenStream {
    let input =
        parse_macro_input!(input with Punctuated::<ExprAssign, Token![,]>::parse_terminated);
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

#[proc_macro]
pub fn url(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input with Punctuated::<Expr, Token![,]>::parse_terminated);
    match url_macro(input) {
        Ok(s) => s.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(StaticFiles, attributes(folder))]
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
