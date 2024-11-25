extern crate proc_macro;

use proc_macro::TokenTree;
use std::str::FromStr;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, LitStr};
use log::debug;

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
pub fn hack(input: TokenStream1) -> TokenStream1 {
    // dbg!(&input);

    let input = parse_macro_input!(input as LitStr);

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

    let stream = quote! {
        self.emitln(#literal);
    };

    // println!("stream = '{}'", &stream);

    stream.into()
}