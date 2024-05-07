use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use rstml::{self, node::Node, Parser, ParserConfig};
use std::{collections::HashSet, fmt::Debug};
use syn::{Expr, Ident, LitStr, Result};

pub fn html_macro(input: TokenStream) -> Result<TokenStream2> {
    let size_hint = input.to_string().len();
    let config = ParserConfig::new()
        .recover_block(true)
        .always_self_closed_elements(HashSet::from([
            "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
            "track", "wbr",
        ]));
    let parser = Parser::new(config);

    let nodes = parser.parse_simple(input)?;
    let buf = Ident::new("__shtml_buf", Span::call_site());
    let mut output = Output {
        buf: buf.clone(),
        static_string: String::new(),
        tokens: vec![],
    };
    nodes
        .into_iter()
        .for_each(|node| render(&mut output, &node));

    let tokens = output.to_token_stream();

    Ok(quote! {
        {
            let mut #buf = String::with_capacity(#size_hint);
            #tokens
            Component { html: #buf }
        }
    })
}

fn render(output: &mut Output, node: &Node) {
    match node {
        Node::Comment(c) => {
            output.push_str("<!--");
            output.push_str(&c.value.value());
            output.push_str("-->");
        }
        Node::Doctype(d) => {
            output.push_str("<!DOCTYPE ");
            output
                .static_string
                .push_str(&d.value.to_token_stream_string());
            output.push_str(">");
        }
        Node::Fragment(n) => {
            for node in &n.children {
                render(output, &node)
            }
        }
        Node::Element(n) => {
            let component_name = match &n.name() {
                rstml::node::NodeName::Path(syn::ExprPath { path, .. }) => match path.get_ident() {
                    Some(ident) => match ident.to_string().get(0..1) {
                        Some(first_letter) => match first_letter.to_uppercase() == first_letter {
                            true => Some(ident),
                            false => None,
                        },
                        None => None,
                    },
                    None => todo!(),
                },
                rstml::node::NodeName::Punctuated(_) => todo!(),
                rstml::node::NodeName::Block(_) => todo!(),
            };
            match component_name {
                Some(fn_name) => {
                    let mut inputs = n
                        .open_tag
                        .attributes
                        .iter()
                        .map(|attr| match attr {
                            rstml::node::NodeAttribute::Block(_) => todo!(),
                            rstml::node::NodeAttribute::Attribute(attr) => {
                                let value = attr.value();

                                quote! { #value }
                            }
                        })
                        .collect::<Vec<_>>();

                    let mut inner_output = Output::new(output.buf.clone());

                    for node in &n.children {
                        render(&mut inner_output, &node);
                    }

                    let buf = inner_output.buf.clone();
                    let inner_tokens = inner_output.to_token_stream();

                    match inner_tokens.is_empty() {
                        false => {
                            let inner_tokens = quote! {
                                {
                                    let mut #buf = String::new();
                                    #inner_tokens
                                    Component { html: #buf }
                                }
                            };

                            inputs.push(inner_tokens);
                        }
                        _ => {}
                    }

                    let tokens = quote! { #fn_name(#(#inputs,)*) };

                    output.push_tokens(tokens);
                }
                None => {
                    output.push_str("<");
                    output.push_str(&n.open_tag.name.to_string());
                    for attr in &n.open_tag.attributes {
                        match attr {
                            rstml::node::NodeAttribute::Block(_) => todo!(),
                            rstml::node::NodeAttribute::Attribute(attr) => {
                                output.static_string.push(' ');
                                output.push_str(&attr.key.to_string());
                                match attr.value_literal_string() {
                                    Some(s) => {
                                        output.push_str("=\"");
                                        output.push_str(&s);
                                        output.push_str("\"");
                                    }
                                    None => match attr.value() {
                                        Some(expr) => {
                                            output.push_str("=\"");
                                            let tokens = expr.to_token_stream();
                                            output.push_tokens(tokens);
                                            output.push_str("\"");
                                        }
                                        None => {
                                            // TODO: bool attr?
                                        }
                                    },
                                }
                            }
                        }
                    }
                    match &n.children.is_empty() {
                        true => match &n.close_tag {
                            Some(tag) => {
                                output.push_str(">");
                                output.push_str("</");
                                output.push_str(&tag.name.to_string());
                                output.push_str(">");
                            }
                            None => {
                                output.push_str("/>");
                            }
                        },
                        false => {
                            output.push_str(">");
                            for child in &n.children {
                                render(output, &child);
                            }

                            match &n.close_tag {
                                Some(tag) => {
                                    output.push_str("</");
                                    output.push_str(&tag.name.to_string());
                                    output.push_str(">");
                                }
                                None => {
                                    output.push_str("/>");
                                }
                            }
                        }
                    }
                }
            }
        }
        Node::Block(n) => {
            let tokens = n.to_token_stream();
            output.push_tokens(tokens);
        }
        Node::Text(n) => output.push_str(&n.value_string()),
        Node::RawText(n) => output.push_str(&n.to_token_stream_string()),
    }
}

#[derive(Debug)]
struct Output {
    buf: Ident,
    static_string: String,
    tokens: Vec<TokenStream2>,
}

impl Output {
    fn new(buf: Ident) -> Self {
        Self {
            buf,
            tokens: vec![],
            static_string: String::new(),
        }
    }

    fn push_str(&mut self, string: &str) {
        self.static_string.push_str(string);
    }

    fn push_tokens(&mut self, tokens: TokenStream2) {
        self.push_expr();
        let buf = &self.buf;
        let tokens = quote! {
            let _ = #tokens.render_to_string(&mut #buf);
        };
        self.tokens.push(tokens);
    }

    fn push_expr(&mut self) {
        if self.static_string.is_empty() {
            return;
        }
        let expr = {
            let output_ident = self.buf.clone();
            let string = LitStr::new(&self.static_string, Span::call_site());
            quote!(#output_ident.push_str(#string);)
        };
        self.static_string.clear();
        self.tokens.push(expr);
    }

    fn to_token_stream(mut self) -> TokenStream2 {
        self.push_expr();
        self.tokens.into_iter().collect()
    }
}

pub fn component_macro(input: Expr) -> Result<TokenStream2> {
    let tokens = match input {
        Expr::Path(syn::ExprPath { path, .. }) => match path.get_ident() {
            Some(ident) => ident.to_string().to_lowercase(),
            None => todo!(),
        },
        _ => todo!(),
    };

    Ok(quote! { #tokens })
}
