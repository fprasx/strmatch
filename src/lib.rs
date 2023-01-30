use std::vec;

use proc_macro_error::{emit_error, proc_macro_error, abort};
use quote::TokenStreamExt;
use quote::{quote, ToTokens};
use syn::parse_macro_input;
use syn::{
    parse::discouraged::Speculative, parse::Parse, Ident, LitByte, LitByteStr, LitChar, LitStr,
    Token,
};

#[proc_macro]
#[proc_macro_error]
pub fn strmatch(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let macro_input = parse_macro_input!(tokens as MacroInput);
    let remainder = macro_input.remainder;
    let literals = macro_input.literals;
    // Note: all literals get trailing commas in their ToToken's impls, so we
    // don't need one here
    quote!([#(#literals)* #remainder @ ..]).into()
}

struct MacroInput {
    literals: Vec<Literal>,
    remainder: Ident,
}

impl Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut literals = vec![];
        // Try to parse a literal
        while let Ok(lit) = input.parse::<Literal>() {
            literals.push(lit);
            // Make sure there is a : token following it
            match input.parse::<Token![:]>() {
                Ok(_) => continue,
                Err(e) => return Err(e),
            }
        }
        let remainder = match input.parse::<Ident>() {
            Ok(rem) => rem,
            Err(e) => return Err(e),
        };
        Ok(MacroInput {
            literals,
            remainder,
        })
    }
}

enum Literal {
    ByteStr { lit: LitByteStr, reps: usize },
    Byte { lit: LitByte, reps: usize },
    Str { lit: LitStr, reps: usize },
    Char { lit: LitChar, reps: usize },
}

fn process_suffix(suffix: &str) -> Result<usize, String> {
    if suffix.is_empty() {
        return Ok(1);
    }
    if suffix.starts_with('x') {
        // We know it starts with x so we can unwrap
        let (_, rest) = suffix.split_once('x').unwrap();
        rest.parse::<usize>()
            .map_err(|_| format!("error parsing {rest} into an integer"))
    } else {
        Err("suffix did not start with `x`".into())
    }
}

impl Parse for Literal {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use Literal::*;

        // Make sure not the join the fork to the input before emitting an
        // error as this will make the error point to the next syntax node.

        // Attempt to parse into a byte string literal
        let fork = input.fork();
        if let Ok(lit) = fork.parse::<LitByteStr>() {
            match process_suffix(lit.suffix()) {
                Ok(reps) => {
                    input.advance_to(&fork);
                    return Ok(ByteStr { lit, reps });
                }
                Err(err) => {
                    emit_error!(input.span(), err)
                }
            }
        }

        // Attempt to parse into a string literal
        let fork = input.fork();
        if let Ok(lit) = fork.parse::<LitStr>() {
            match process_suffix(lit.suffix()) {
                Ok(reps) => {
                    input.advance_to(&fork);
                    return Ok(Str { lit, reps });
                }
                Err(err) => {
                    emit_error!(input.span(), err)
                }
            }
        }

        // Attempt to parse into a byte literal
        let fork = input.fork();
        if let Ok(lit) = fork.parse::<LitByte>() {
            match process_suffix(lit.suffix()) {
                Ok(reps) => {
                    input.advance_to(&fork);
                    return Ok(Byte { lit, reps });
                }
                Err(err) => {
                    emit_error!(input.span(), err)
                }
            }
        }

        // Attempt to parse into a char
        let fork = input.fork();
        if let Ok(lit) = fork.parse::<LitChar>() {
            match process_suffix(lit.suffix()) {
                Ok(reps) => {
                    input.advance_to(&fork);
                    return Ok(Char { lit, reps });
                }
                Err(err) => {
                    emit_error!(input.span(), err)
                }
            }
        }
        // None of the parsers succeeded
        Err(syn::Error::new(input.span(), "failed to parse input as byte string literal, string literal, byte literal, or character"))
    }
}

impl ToTokens for Literal {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Literal::ByteStr { lit, reps } => {
                for _ in 0..*reps {
                    let bytes = lit.value();
                    tokens.append_terminated(bytes.iter(), quote!(,))
                }
            }
            Literal::Byte { lit, reps } => {
                for _ in 0..*reps {
                    let byte = lit.value();
                    tokens.append_all(quote!(#byte,))
                }
            }
            Literal::Str { lit, reps } => {
                for _ in 0..*reps {
                    let string = lit.value();
                    tokens.append_terminated(string.chars(), quote!(,))
                }
            }
            Literal::Char { lit, reps } => {
                for _ in 0..*reps {
                    let char = lit.value();
                    tokens.append_all(quote!(#char,))
                }
            }
        }
    }
}
