extern crate proc_macro;

use proc_macro::TokenTree;
use std::str::FromStr;
use proc_macro2::{Group, Span};
use quote::quote;
use syn::{parse_macro_input, LitStr, Token};
use log::debug;
use quote::__private::ext::RepToTokensExt;
use syn::parse::{Parse, ParseStream};

type TokenStream1 = proc_macro::TokenStream;
type TokenStream2 = proc_macro2::token_stream::TokenStream;

type TokenTree1 = proc_macro::TokenTree;
type TokenTree2 = proc_macro2::TokenTree;

/// Emit hack assembly
/// - Takes a raw multiline string.
/// - In each line it removes leading and trailing whitespace
/// - does not append a newline at the end
/// - calls self.emitln with the generated string
#[proc_macro]
pub fn emit_hack(input: TokenStream1) -> TokenStream1 {
    // dbg!(&input);
    let input = parse_macro_input!(input as LitStr);

    let literal = trim_hack_str(input);

    let stream = quote! {
        self.emitln(#literal);
    };

    // println!("stream = '{}'", &stream);

    stream.into()
}

fn trim_hack_str(input: LitStr) -> LitStr {
    let input = input.value();
    let mut trimmed = Vec::with_capacity(input.lines().count());
    let lines = input.lines();
    for line in lines {
        let line = line.trim();
        if line.len() == 0 {
            continue;
        }
        let mut line = line.to_string();
        line.push_str("\n");
        trimmed.push(line);
    }

    let all = trimmed.iter()
        .map(|s| s.chars())
        .flatten()
        .collect::<String>();

    let all = all.trim_end();

    let literal = LitStr::new(all, Span::call_site());

    literal
}

/// Strip a multiline  &str
/// - Takes a raw multiline string.
/// - In each line it removes leading and trailing whitespace
/// - does not append a newline at the end
/// - Inserts the stripped &str
#[proc_macro]
pub fn hack_str(input: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(input as LitStr);
    let input = trim_hack_str(input);

    let stream = quote! {
        #input
    };

    // dbg!(&stream);
    return stream.into();
}

struct HackFmt {
    string_literal: LitStr,
    rest_of_tokens: TokenStream2,
}

impl Parse for HackFmt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let string_literal = input.parse::<LitStr>()?;
        let rest_of_tokens = input.parse::<TokenStream2>()?;

        Ok(HackFmt {
            string_literal,
            rest_of_tokens,
        })
    }
}


fn construct_hack_fmt(input: TokenStream1) -> TokenStream2 {
    let input = syn::parse::<HackFmt>(input)
        .expect("failed to parse hack format args");
    let fmt = input.string_literal;
    let rest = input.rest_of_tokens;

    let fmt = trim_hack_str(fmt);

    let stream = quote! {
        format!("{0}",format_args!{#fmt #rest})
    };

    stream.into()
}

/// like format!, except the first argument is stripped of whitespace
/// - In each line it removes leading and trailing whitespace
/// - does not append a newline at the end
/// - calls to format with the &str and any other args passed
#[proc_macro]
pub fn fmt_hack(input: TokenStream1) -> TokenStream1 {
    let stream = construct_hack_fmt(input);

    // println!("{}",&stream);

    return stream.into();
}

/// Similar to format!, except the result is passed to self.emitln(&str)
/// - In each line it removes leading and trailing whitespace
/// - does not append a newline at the end
/// - calls to format! with the &str and any other args passed
/// - passes the result to `self.emitln` function
#[proc_macro]
pub fn emit_fmt_hack(input: TokenStream1) -> TokenStream1 {
    let stream = construct_hack_fmt(input);

    let stream = quote! {
        self.emitln(& #stream);
    };

    // println!("{}", stream);

    stream.into()
}