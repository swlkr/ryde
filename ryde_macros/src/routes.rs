use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::HashSet;
use syn::{
    parse::Parse, punctuated::Punctuated, Expr, ExprCall, ExprLit, ExprMethodCall, ExprPath,
    ExprTuple, Ident, ItemFn, Lit, LitStr, Result, Token,
};

pub fn router_macro(input: ItemFn) -> Result<TokenStream> {
    let mut parts: Vec<(String, Ident)> = vec![];

    input.block.stmts.iter().for_each(|stmt| match stmt {
        syn::Stmt::Expr(expr, _) => router(&Box::new(expr.clone()), None, &mut parts),
        _ => {}
    });

    let helpers = parts
        .iter()
        .map(|(s, handler)| {
            let ident = Ident::new(&format!("{}_path", handler), Span::call_site());
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
        .collect::<Vec<_>>();

    Ok(quote! {
        #input

        #(#helpers)*
    })
}

fn router(expr: &Box<Expr>, lit_str: Option<&LitStr>, output: &mut Vec<(String, Ident)>) {
    match &**expr {
        Expr::MethodCall(ExprMethodCall {
            receiver,
            method,
            args,
            ..
        }) => {
            // 1. look for LitStr route
            match method.to_string().as_str() {
                "route" => {
                    match args.iter().collect::<Vec<_>>()[..] {
                        [Expr::Lit(ExprLit {
                            lit: Lit::Str(lit_str),
                            ..
                        }), Expr::Call(ExprCall { func, args, .. })] => {
                            match &**func {
                                Expr::Path(ExprPath { path, .. }) => {
                                    let ident = path.get_ident();
                                    // 2. look for get, post, put, delete, patch, trace, head
                                    match find_route(ident, args, lit_str) {
                                        Some(route) => output.push(route),
                                        None => {}
                                    }
                                }
                                _ => unimplemented!(),
                            }
                        }
                        [Expr::Lit(ExprLit {
                            lit: Lit::Str(lit_str),
                            ..
                        }), Expr::MethodCall(ExprMethodCall {
                            receiver,
                            method,
                            args,
                            ..
                        })] => {
                            match find_route(Some(method), args, lit_str) {
                                Some(route) => {
                                    output.push(route);
                                }
                                None => {}
                            };
                            router(receiver, Some(lit_str), output);
                        }
                        _ => todo!(),
                    }
                }
                _ => {}
            }
            router(receiver, lit_str, output);
        }
        Expr::Call(ExprCall { func, args, .. }) => match &**func {
            Expr::Path(ExprPath { path, .. }) => {
                let ident = path.get_ident();
                match lit_str {
                    Some(lit_str) => match find_route(ident, args, lit_str) {
                        Some(route) => output.push(route),
                        None => {}
                    },
                    None => {}
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!("Only Router::new()... supported for now"),
    }
}

fn find_route(
    ident: Option<&Ident>,
    args: &Punctuated<Expr, syn::token::Comma>,
    lit_str: &LitStr,
) -> Option<(String, Ident)> {
    match ident {
        Some(ident) => match ident.to_string().as_str() {
            "get" | "post" | "put" | "patch" | "delete" | "trace" | "head" => {
                // 3. finally get name of handler ident (only idents are supported)
                match args.last() {
                    Some(&Expr::Path(ExprPath { ref path, .. })) => match path.get_ident() {
                        Some(ident) => Some((lit_str.value(), ident.clone())),
                        None => unimplemented!("Only fn handlers are supported."),
                    },
                    Some(_) | None => unimplemented!("Only fn handlers are supported."),
                }
            }
            _ => None,
        },
        None => todo!("Failed when looking for http method ident"),
    }
}

pub fn routes_macro(input: StateRouter) -> Result<TokenStream> {
    let parts: Vec<(&Lit, Vec<Ident>, &Expr)> = input
        .routes
        .iter()
        .filter_map(|ExprTuple { elems, .. }| {
            let mut iter = elems.iter();
            let (Some(Expr::Lit(ExprLit { lit, .. })), Some(expr)) = (iter.nth(0), iter.nth(0))
            else {
                return None;
            };

            parts(expr, lit)
        })
        .collect();

    let routes = parts.iter().map(|(lit, _ident, expr)| {
        quote! { .route(#lit, #expr) }
    });

    let helpers = parts
        .iter()
        .flat_map(|(lit, handlers, _expr)| {
            handlers
                .iter()
                .map(|handler| handler.to_string())
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

    let tokens = quote! {
        fn routes() -> axum::Router<#generic> {
            use axum::routing::{get, post, put, patch, head, trace};

            axum::Router::new()#(#routes)*
        }

        #(#helpers)*
    };

    Ok(tokens)
}

fn parts<'a>(expr: &'a Expr, lit: &'a Lit) -> Option<(&'a Lit, Vec<Ident>, &'a Expr)> {
    let idents = handlers(&expr);
    if idents.is_empty() {
        None
    } else {
        Some((lit, idents, expr))
    }
}

fn handlers(expr: &Expr) -> Vec<Ident> {
    match expr {
        Expr::Call(ExprCall { args, .. }) => handler(args),
        Expr::MethodCall(ExprMethodCall { receiver, args, .. }) => {
            let mut idents = handler(args);
            let rest = handlers(&receiver);
            idents.extend(rest);

            idents
        }
        _ => vec![],
    }
}

fn handler(args: &Punctuated<Expr, syn::token::Comma>) -> Vec<Ident> {
    args.iter()
        .filter_map(|arg| match arg {
            Expr::Path(ExprPath { path, .. }) => path.get_ident().cloned(),
            _ => None,
        })
        .collect::<Vec<Ident>>()
}

pub fn url_macro(Url { url, path }: Url) -> Result<TokenStream> {
    let fn_name = Ident::new(&format!("{}_path", url.to_string()), Span::call_site());

    Ok(quote! {
        {
            let _ = &#url;
            #fn_name(#path)
        }
    })
}

pub struct Url {
    url: Ident,
    path: Punctuated<Expr, Token![,]>,
}

impl Parse for Url {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let url = input.parse::<syn::Ident>()?;
        let _comma: Option<Token![,]> = input.parse()?;
        let path = Punctuated::parse_terminated(input)?;

        Ok(Self { url, path })
    }
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
