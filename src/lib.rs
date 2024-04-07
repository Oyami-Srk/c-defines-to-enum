#![feature(proc_macro_expand)]
#![feature(let_chains)]

extern crate proc_macro;

use proc_macro::{TokenStream, TokenTree};
use std::collections::HashMap;

use syn::{parse_macro_input, Meta, Path, Lit, Macro, LitStr};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Comma};

struct CDefinesToEnumInput {
    pub name: Path,
    // pub data_type: String,
    pub content: String,

    pub remove_suffix: String,
    pub remove_prefix: String,
    pub to_upper: bool,
    pub to_lower: bool,
}

fn parse_lit_to_string(lit: &Lit) -> String {
    match lit {
        Lit::Str(v) => v.value(),
        _ => panic!("Lit {} is not a String.", lit.to_token_stream().to_string())
    }
}

fn parse_lit_to_bool(lit: &Lit) -> bool {
    match lit {
        Lit::Bool(v) => v.value(),
        _ => panic!("Lit {} is not a bool.", lit.to_token_stream().to_string())
    }
}

enum MetaOrMacro {
    Meta(Meta),
    Macro(Macro),
}

impl Parse for MetaOrMacro {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(meta) = input.parse::<Meta>() {
            Ok(MetaOrMacro::Meta(meta))
        } else if let Ok(macro_) = input.parse::<Macro>() {
            Ok(MetaOrMacro::Macro(macro_))
        } else {
            panic!("Unknown token.");
        }
    }
}

impl Parse for CDefinesToEnumInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name_values: Punctuated<MetaOrMacro, Comma> =
            input.parse_terminated(MetaOrMacro::parse).unwrap();
        let mut attrs: HashMap<String, Lit> = HashMap::new();
        let mut enum_name: Option<Path> = None;

        name_values.into_iter().for_each(|v| {
            match v {
                MetaOrMacro::Meta(Meta::Path(name)) => { enum_name = Some(name) }
                MetaOrMacro::Meta(Meta::List(_)) => {}
                MetaOrMacro::Meta(Meta::NameValue(v)) => {
                    let key = v.path.get_ident().into_token_stream().to_string();
                    attrs.insert(key, v.lit);
                }
                MetaOrMacro::Macro(macro_) => {
                    let tks: proc_macro::TokenStream = macro_.to_token_stream().into();
                    let s = TokenStream::from(tks).expand_expr().unwrap();
                    let s = s.into_iter().collect::<Vec<TokenTree>>();
                    let value = match &s.get(0) {
                        Some(TokenTree::Literal(lit)) => lit.clone(),
                        _ => panic!("Macro expanded is not a literal.")
                    };
                    // attrs.insert("content", value.into());
                    // println!("{}", quote!(#value));
                    let value = proc_macro::TokenTree::from(value);
                    let value = TokenStream::from(value);
                    let value: LitStr = parse_macro_input::parse::<LitStr>(value)
                        .expect("Macro expanded is not a string literal");
                    attrs.insert("content".into(), value.into());
                }
            }
        });

        // let data_type = attrs.get("data_type").map(parse_lit_to_string).unwrap_or("usize".into());
        Ok(CDefinesToEnumInput {
            name: enum_name.expect("Not specified enum name"),
            // data_type,
            content: attrs.get("content").map(parse_lit_to_string).expect("Content is missing."),
            remove_suffix: attrs.get("remove_suffix").map(parse_lit_to_string).unwrap_or("".into()),
            remove_prefix: attrs.get("remove_prefix").map(parse_lit_to_string).unwrap_or("".into()),
            to_upper: attrs.get("to_upper").map(parse_lit_to_bool).unwrap_or(false),
            to_lower: attrs.get("to_lower").map(parse_lit_to_bool).unwrap_or(false),
        })
    }
}

// From https://github.com/jiegec/cdefines
fn parse_value(value: &str) -> Option<usize> {
    if value.starts_with("0x") {
        usize::from_str_radix(value.trim_start_matches("0x"), 16).ok()
    } else if value.starts_with("0b") {
        usize::from_str_radix(value.trim_start_matches("0b"), 2).ok()
    } else if value.starts_with("0") {
        usize::from_str_radix(value.trim_start_matches("0"), 8).ok()
    } else {
        usize::from_str_radix(value, 10).ok()
    }
}

fn parse_c_define_strings(input: &CDefinesToEnumInput) -> (HashMap<String, usize>, bool) {
    let content = &input.content;
    let mut symbols: HashMap<String, usize> = HashMap::new();
    let mut has_dup = false;

    let normalize_name = |name: &str| -> String {
        let name = name.strip_prefix(&input.remove_prefix).unwrap_or(name);
        let name = name.strip_suffix(&input.remove_suffix).unwrap_or(name);
        let name = if input.to_lower {
            name.to_lowercase()
        } else if input.to_upper {
            name.to_uppercase()
        } else {
            name.to_string()
        };
        name
    };

    content.lines().filter_map(|line| {
        let mut tokens = line.split_whitespace();
        if let Some(first) = tokens.next() &&
            first == "#define" {
            Some((tokens.next().expect("missing define name"), tokens.next().expect("missing define value")))
        } else {
            None
        }
    }).for_each(|(name, value)| {
        let name = normalize_name(name);

        let v = if let Some(value) = parse_value(value) {
            value
        } else {
            let value = normalize_name(value);
            has_dup = true;
            if let Some(v) = symbols.get(value.as_str()) {
                *v
            } else {
                panic!("{} is defined ahead of {}", name, value);
            }
        };
        symbols.insert(name.to_string(), v);
    });

    if !has_dup {
        let mut values = symbols.values().map(|v| *v).collect::<Vec<_>>();
        if values.len() != {
            values.sort();
            values.dedup();
            values.len()
        } {
            has_dup = true;
        }
    }
    (symbols, has_dup)
}

#[proc_macro]
pub fn parse_c_defines_to_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as CDefinesToEnumInput);
    let enum_name = &input.name;
    let (symbols, has_dup) = parse_c_define_strings(&input);
    let enum_content = symbols.iter().map(|(name, value)| {
        let ident = format_ident!("{}", name);
        if has_dup {
            quote!(#ident,)
        } else {
            quote!(#ident = #value,)
        }
    }).collect::<Vec<_>>();
    let enum_from_content = symbols.iter().map(|(name, value)| {
        let ident = format_ident!("{}", name);
        quote!(#value => Ok(Self::#ident),)
    }).collect::<Vec<_>>();
    let enum_convert = if has_dup {
        let matches =
            symbols.iter().map(|(name, value)| {
                let ident = format_ident!("{}", name);
                quote!(Self::#ident => #value,)
            }).collect::<Vec<_>>();
        quote! {
            match self {
                #(#matches)*
            }
        }
    } else {
        quote!(self as usize)
    };

    let token_stream = quote! {
        #[repr(usize)]
        #[derive(Copy, Clone, Debug)]
        #[allow(non_camel_case_types)]
        pub enum #enum_name {
            #(#enum_content)*
        }

        impl TryFrom<usize> for #enum_name {
            type Error = ();

            fn try_from(value: usize) -> Result<Self, Self::Error> {
                match value {
                    #(#enum_from_content)*
                    _ => Err(()),
                }
            }
        }

        impl Into<usize> for #enum_name {
            fn into(self) -> usize {
                #enum_convert
            }
        }
    };

    token_stream.into()
}
