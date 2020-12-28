#![feature(proc_macro, proc_macro_lib)]
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use quote::quote;

/// This crate is a custom implementation for Debug
/// In fact, the problem which tries to solve is that
/// we aren't interested in full recursive println of
/// instructions.
///
/// Remember, `OP_BLOCK(_, nested block)`
/// we are not interested in the nested block
use proc_macro::TokenStream;
use syn::{Body, Ident, Variant, VariantData};

#[proc_macro_derive(CustomDisplay)]
pub fn custom_display(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_macro_input(&s).unwrap();

    // Build the impl
    let name = &ast.ident;
    let gen = match ast.body {
        Body::Enum(ref variants) => impl_display(name, variants),
        Body::Struct(_) => panic!("Macro only for enums"),
    };

    // Return the generated impl
    gen.parse().unwrap()
}

fn impl_display(name: &Ident, variants: &[Variant]) -> quote::Tokens {
    let arms = arms(&name, variants);
    quote! {
        impl std::fmt::Display for #name {
             fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                 quote! {
                     match *self {
                         #(#arms)*
                         _=> unreachable!()
                        //write!(f, "({}, {})", "test", "test");
                    }
                 }
            }
        }
    }
}

fn arms(name: &Ident, variants: &[Variant]) -> Vec<quote::Tokens> {
    let mut result = Vec::new();

    for (idx, variant) in variants.iter().enumerate() {
        let id = &variant.ident;
        let new = match variant.data {
            VariantData::Unit => quote! {
                #id => write!(f, "{}", #id)
            },
            VariantData::Tuple(ref _fields) => quote! {
                #id => write!(f, "{}", #id)
            },
            _ => panic!("structs not supported"),
        };

        result.push(new);
    }

    result
}
