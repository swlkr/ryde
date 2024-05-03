use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::HashSet;
use syn::{
    parse::Parse, punctuated::Punctuated, Expr, ExprCall, ExprLit, ExprMethodCall, ExprPath,
    ExprTuple, Ident, Lit, Result, Token,
};

pub fn routes_macro(input: StateRouter) -> Result<TokenStream> {
    let parts: Vec<(&Lit, Vec<(Ident, Ident)>)> = input
        .routes
        .iter()
        .filter_map(|ExprTuple { elems, .. }| {
            let mut iter = elems.iter();
            let (Some(Expr::Lit(ExprLit { lit, .. })), Some(expr)) = (iter.nth(0), iter.nth(0))
            else {
                return None;
            };

            match expr {
                Expr::Call(ExprCall { func, args, .. }) => {
                    let handler = handler(args);
                    let fn_name = match &**func {
                        Expr::Path(ExprPath { path, .. }) => path
                            .get_ident()
                            .expect("fn name should be an identifier")
                            .clone(),
                        _ => panic!("fn name should be an identifier"),
                    };

                    Some(vec![(lit, vec![(fn_name, handler)])])
                }
                Expr::MethodCall(ExprMethodCall {
                    receiver,
                    method,
                    args,
                    ..
                }) => {
                    let handler = handler(args);
                    let mut result: Vec<(Ident, Ident)> = vec![(method.clone(), handler)];
                    result.extend(method_router(&receiver));

                    Some(vec![(lit, result)])
                }
                _ => None,
            }
        })
        .flatten()
        .collect();

    let routes = parts
        .iter()
        .map(|(lit, method_router)| {
            let tokens = method_router
                .iter()
                .map(|(fn_name, handler)| quote! { #fn_name(#handler) })
                .collect::<Vec<_>>();

            quote! {
                .route(#lit, #(#tokens).*)
            }
        })
        .collect::<Vec<_>>();

    let helpers = parts
        .iter()
        .flat_map(|(lit, method_router)| {
            method_router
                .iter()
                .map(|(_method, handler)| handler.to_string())
                .collect::<HashSet<_>>()
                .into_iter()
                .map(|x| {
                    let ident = Ident::new(&format!("{}_path", x), Span::call_site());
                    let s = match lit {
                        syn::Lit::Str(s) => s.value(),
                        _ => panic!("route needs to a string"),
                    };
                    let format_string = s
                        .split("/")
                        .map(|x| match x.starts_with(":") {
                            true => "{}",
                            false => x,
                        })
                        .collect::<Vec<_>>()
                        .join("/");
                    let fn_args = s
                        .split("/")
                        .filter(|x| x.starts_with(":"))
                        .map(|s| {
                            let ident = Ident::new(&s.replace(":", ""), Span::call_site());

                            quote! { #ident: impl std::fmt::Display }
                        })
                        .collect::<Vec<_>>();
                    let format_args = s
                        .split("/")
                        .filter(|x| x.starts_with(":"))
                        .map(|s| Ident::new(&s.replace(":", ""), Span::call_site()))
                        .collect::<Vec<_>>();

                    quote! {
                        fn #ident(#(#fn_args,)*) -> String {
                            format!(#format_string, #(#format_args,)*)
                        }
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let generic = match input.state {
        Some(tp) => quote! { #tp },
        None => quote! { () },
    };

    Ok(quote! {
        fn routes() -> axum::Router<#generic> {
            use axum::routing::{get, post, put, patch, head, trace};

            axum::Router::new()#(#routes)*
        }

        #(#helpers)*
    })
}

fn method_router(expr: &Expr) -> Vec<(Ident, Ident)> {
    match expr {
        Expr::Call(ExprCall { func, args, .. }) => {
            let method_name = match &**func {
                Expr::Path(ExprPath { path, .. }) => path
                    .get_ident()
                    .cloned()
                    .expect("fn name should be an identifier"),
                _ => unimplemented!(),
            };
            let handler = handler(args);

            vec![(method_name, handler)]
        }
        Expr::MethodCall(ExprMethodCall {
            receiver,
            method,
            args,
            ..
        }) => {
            let handler = handler(args);

            let mut result = vec![(method.clone(), handler)];
            result.extend(method_router(&receiver));
            result
        }
        _ => unimplemented!(),
    }
}

fn handler(args: &Punctuated<Expr, syn::token::Comma>) -> Ident {
    match args.first() {
        Some(Expr::Path(ExprPath { path, .. })) => path
            .get_ident()
            .cloned()
            .expect("only named fns as handlers are supported"),

        _ => unimplemented!(),
    }
}

pub fn url_macro(input: Punctuated<Expr, Token![,]>) -> Result<TokenStream> {
    let Some(Expr::Path(ExprPath { path, .. })) = input.first() else {
        panic!("first argument should be an handler fn name");
    };
    let Some(ident) = path.get_ident() else {
        panic!("first argument should be an ident");
    };
    let fn_name = Ident::new(&format!("{}_path", ident.to_string()), Span::call_site());
    let rest = input.iter().skip(1).collect::<Vec<_>>();

    Ok(quote! {
        #fn_name(#(#rest,)*)
    })
}

pub struct StateRouter {
    routes: Punctuated<ExprTuple, Token![,]>,
    state: Option<syn::TypePath>,
}

impl Parse for StateRouter {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let routes = Punctuated::parse_separated_nonempty(input)?;

        let state = match input.parse::<syn::Ident>().ok() {
            Some(_) => input.parse::<syn::TypePath>().ok(),
            None => None,
        };

        Ok(Self { state, routes })
    }
}
