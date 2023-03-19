use std::vec;

use proc_macro_error::{abort, proc_macro_error};
use quote::TokenStreamExt;
use quote::{quote, ToTokens};
use syn::{bracketed, parse_macro_input};
use syn::{parse::Parse, Ident, LitByte, LitByteStr, LitChar, LitStr, Token};

/// `strmatch!` makes validating and extracting parts of
/// strings easier. It works by converting your query into a slice pattern,
/// hopefully allowing to optimizer to take a stab at your queries.
///
/// # Usage:
///
/// ```rust
/// // Convert to bytes so we can use slice pattern matching.
/// let str = "one twotwo threethreethree";
///
/// // This `as_bytes` call is completely free!
/// // It's just a transmute under the hood.
/// match str.as_bytes() {
///     // We can start off by matching an empty string
///     strmatch!() => {}
///
///     // Ignore one character ...
///     strmatch!(_) => {}
///
///     // Or take it!
///     strmatch!(mine_now) => {}
///
///     // Match a literal ...
///     strmatch!('x') => {}
///     strmatch!("xyz") => {}
///
///     // And match repeats!
///     strmatch!("one" _ "two"x2  _ "three"x3) => {}
///
///     // Bracketed patterns can be the last term of a pattern.
///     // Ignore everything past "one"
///     strmatch!("one" [_]) => {}
///
///     // Or give it a name :)
///     strmatch!("one" _ [hellooo]) => {
///         assert_eq!(hellooo, b"twotwo threethreethree");
///     }
///
///     // We can combine patterns however we want!
///     strmatch!("one" ' ' "two"x2 space "three"x2 [rest]) => {
///         assert_eq!(space, &b' ');
///         assert_eq!(rest, b"three");
///     }
///
///     _ => println!("Macros are fun :p"),
/// }
/// ```
#[proc_macro]
#[proc_macro_error]
pub fn strmatch(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if tokens.is_empty() {
        return quote!([]).into();
    }

    let macro_input = parse_macro_input!(tokens as MacroInput);
    let end = macro_input.end;
    let literals = macro_input.literals;
    if let Some(end) = end {
        quote!([#(#literals)* #end]).into()
    } else {
        quote!([#(#literals)*]).into()
    }
}

struct MacroInput {
    literals: Vec<Capture>,
    end: Option<EndCapture>,
}

impl Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut literals = vec![];
        // Try to parse a literal
        while let Ok(lit) = input.parse::<Capture>() {
            literals.push(lit);
        }
        if input.is_empty() {
            return Ok(MacroInput {
                literals,
                end: None,
            });
        }
        let inner;
        let _ = bracketed!(inner in input);
        match inner.parse::<EndCapture>() {
            Err(e) => Err(e),
            Ok(end) => Ok(MacroInput {
                literals,
                end: Some(end),
            }),
        }
    }
}

/// `EndCapture` is meant to represent the last capture that grabs all
/// remaining characters, as in [, , , end_capture @ ..] or [, , , _]
enum EndCapture {
    Ident(Ident),
    Underscore,
}

impl Parse for EndCapture {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![_]) {
            input.parse::<Token![_]>().map(|_| EndCapture::Underscore)
        } else if lookahead.peek(Ident) {
            input.parse::<Ident>().map(EndCapture::Ident)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for EndCapture {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            EndCapture::Ident(ident) => tokens.append_all(quote!(#ident @ ..,)),
            EndCapture::Underscore => tokens.append_all(quote!(..,)),
        }
    }
}

/// Any capture that does grab and arbitrary number of tokens.
/// Each of the string-style captures can also have a number of repetitions
/// provided that dictates how many times the proc-macro includes them.
/// These are possible captures of each type
/// `ByteStr`:    b"abc"x2 --expands to-> [b'a', b'b', b'c', b'a', b'b', b'c',]
/// `Byte`:       b'b'x2   --expands to-> [b'b', b'b',]
/// `Str`:        "abc!"x2 --expands to-> [b'a', b'b', b'c', b'a', b'b', b'c',]
/// `Char`:       'c'x2    --expands to-> ['c', 'c',]
/// `Ident`:      abc      --expands to-> [abc @ _,]
/// `Underscore`: _        --expands to-> [_,]
enum Capture {
    ByteStr { lit: LitByteStr, reps: usize },
    Byte { lit: LitByte, reps: usize },
    Str { lit: LitStr, reps: usize },
    Char { lit: LitChar, reps: usize },
    Ident(Ident),
    Underscore,
}

// Return the number of repetitionss from a suffix
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

impl Parse for Capture {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            input.parse().map(Capture::Ident)
        } else if lookahead.peek(Token![_]) {
            input.parse::<Token![_]>().map(|_| Capture::Underscore)
        } else if lookahead.peek(LitByte) {
            match input.parse::<LitByte>() {
                Ok(lit) => {
                    let reps = match process_suffix(lit.suffix()) {
                        Ok(reps) => reps,
                        Err(e) => abort!(lit.span(), e),
                    };
                    return Ok(Capture::Byte { lit, reps });
                }
                Err(_) => unreachable!(), // we checked with lookahead
            }
        } else if lookahead.peek(LitByteStr) {
            match input.parse::<LitByteStr>() {
                Ok(lit) => {
                    let reps = match process_suffix(lit.suffix()) {
                        Ok(reps) => reps,
                        Err(e) => abort!(lit.span(), e),
                    };
                    return Ok(Capture::ByteStr { lit, reps });
                }
                Err(_) => unreachable!(), // we checked with lookahead
            }
        } else if lookahead.peek(LitChar) {
            match input.parse::<LitChar>() {
                Ok(lit) => {
                    let reps = match process_suffix(lit.suffix()) {
                        Ok(reps) => reps,
                        Err(e) => abort!(lit.span(), e),
                    };
                    return Ok(Capture::Char { lit, reps });
                }
                Err(_) => unreachable!(), // we checked with lookahead
            }
        } else if lookahead.peek(LitStr) {
            match input.parse::<LitStr>() {
                Ok(lit) => {
                    let reps = match process_suffix(lit.suffix()) {
                        Ok(reps) => reps,
                        Err(e) => abort!(lit.span(), e),
                    };
                    return Ok(Capture::Str { lit, reps });
                }
                Err(_) => unreachable!(), // we checked with lookahead
            }
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for Capture {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Capture::ByteStr { lit, reps } => {
                for _ in 0..*reps {
                    let bytes = lit.value();
                    tokens.append_terminated(bytes.iter(), quote!(,))
                }
            }
            Capture::Byte { lit, reps } => {
                for _ in 0..*reps {
                    let byte = lit.value();
                    tokens.append_all(quote!(#byte,))
                }
            }
            Capture::Str { lit, reps } => {
                for _ in 0..*reps {
                    let string = lit.value();
                    // We want to display in byte literal form
                    let chars = string.as_bytes();
                    tokens.append_terminated(chars, quote!(,))
                }
            }
            Capture::Char { lit, reps } => {
                for _ in 0..*reps {
                    // Display as a byte literal
                    let char = lit.value() as u8;
                    tokens.append_all(quote!(#char,))
                }
            }
            Capture::Ident(ident) => tokens.append_all(quote!(#ident,)),
            Capture::Underscore => tokens.append_all(quote!(_,)),
        }
    }
}
