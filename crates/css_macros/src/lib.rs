use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, punctuated::Punctuated, Expr, ExprLit, Lit, Result, Token};

#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input with Punctuated::<Expr, Token![,]>::parse_terminated);
    match css_macro(input) {
        Ok(s) => s.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn css_macro(input: Punctuated<Expr, Token![,]>) -> Result<TokenStream2> {
    let lines = input
        .iter()
        .map(|expr| match expr {
            Expr::Lit(ExprLit { lit, .. }) => match lit {
                Lit::Str(s) => s.value(),
                _ => todo!(),
            },
            _ => todo!(),
        })
        .map(|s| (class_name(&s), s))
        .collect::<Vec<_>>();

    let css = generate_css(&lines);

    let classes = lines
        .iter()
        .map(|line| line.0.replace(r"\:", ":").clone())
        .collect::<Vec<_>>()
        .join(" ");

    Ok(quote! { (#classes, vec![#(#css,)*]) })
}

fn class_name(rule: &str) -> String {
    let class_name = rule
        .replace(" ", "")
        .replace(":var(--", "-")
        .replace(":", "-")
        .trim_end_matches(')')
        .replace(":", r"\:")
        .to_string();

    replace_repeated_strings(&class_name)
}

fn generate_css(rules: &Vec<(String, String)>) -> Vec<String> {
    rules
        .iter()
        .map(|(class, rule)| {
            let parts: Vec<_> = rule.split(":").map(|s| s.trim()).collect();
            match parts[..] {
                [property, value] => {
                    format!(".{}{{{}:{};}}", class, property, value)
                }
                _ => {
                    let mods: Vec<_> = parts
                        .iter()
                        .take(parts.len() - 2)
                        .map(|m| mod_(*m))
                        .collect();
                    let pseudos = mods
                        .iter()
                        .filter_map(pseudo)
                        .rev()
                        .collect::<Vec<_>>()
                        .join("");
                    let class = format!("{}{}", class, pseudos);
                    let rule = parts
                        .iter()
                        .skip(parts.len() - 2)
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(":");
                    let mut css = format!(".{}{{{};}}", class, rule);
                    let media = mods.iter().filter_map(media).collect::<Vec<_>>();
                    for query in media.iter().rev() {
                        css = surround(&css, query);
                    }
                    css
                }
            }
        })
        .collect::<Vec<_>>()
}

fn replace_repeated_strings(s: &str) -> String {
    let mut parts: Vec<&str> = s.split('-').collect();
    let mut seen = HashSet::new();
    parts.retain(|item| seen.insert(*item));
    parts.join("-")
}

enum Mod<'a> {
    Pseudo(String),
    Media(&'a str),
}

fn surround(s: &str, with: &str) -> String {
    format!("{}{{{}}}", with, s)
}

fn mod_(part: &str) -> Mod {
    match part {
        "dark" => Mod::Media("prefers-color-scheme:dark"),
        "sm" => Mod::Media("min-width:640px"),
        "md" => Mod::Media("min-width:768px"),
        "lg" => Mod::Media("min-width:1024px"),
        "xl" => Mod::Media("min-width:1280px"),
        "2xl" => Mod::Media("min-width:1536px"),
        "portrait" => Mod::Media("orientation:portrait"),
        "landscape" => Mod::Media("orientation:landscape"),
        "motion-safe" => Mod::Media("prefers-reduced-motion:no-prefence"),
        "motion-reduce" => Mod::Media("prefers-reduced-motion:reduce"),
        part => {
            if part.starts_with("aria-") {
                Mod::Pseudo(format!("[{}=\"true\"]", part))
            } else {
                Mod::Pseudo(part.into())
            }
        }
    }
}

fn pseudo<'a>(mod_: &'a Mod) -> Option<String> {
    match mod_ {
        Mod::Pseudo(p) => Some(format!(":{}", p)),
        Mod::Media(_) => None,
    }
}

fn media<'a>(mod_: &'a Mod) -> Option<String> {
    match mod_ {
        Mod::Pseudo(_) => None,
        Mod::Media(m) => Some(format!("@media({})", m)),
    }
}
