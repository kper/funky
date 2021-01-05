extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// This crate is a custom implementation for Debug
/// In fact, the problem which tries to solve is that
/// we aren't interested in full recursive println of
/// instructions.
///
/// Remember, `OP_BLOCK(_, nested block)`
/// we are not interested in the nested block

#[proc_macro_derive(CustomDisplay)]
pub fn custom_display(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    // Parse the string representation
    //let ast = syn::parse_macro_input(&s).unwrap();

    // Build the impl
    let name = &ast.ident;

    match ast.data {
        Data::Enum(ref data) => {
            let mut impls = Vec::new();
            for variant in &data.variants {
                let id = &variant.ident;

                let name = match &variant.fields {
                    Fields::Unnamed(fields) => match fields.unnamed.len() {
                        1 => {
                            quote! {
                                #name::#id(_) => write!(f, "{}\n", stringify!(#id)),
                            }
                        }
                        2 => {
                            quote! {
                                #name::#id(_,_) => write!(f, "{}\n", stringify!(#id)),
                            }
                        }
                        3 => {
                            quote! {
                                #name::#id(_,_,_) => write!(f, "{}\n", stringify!(#id)),
                            }
                        }
                        _ => panic!(
                            "Enums with fields with more than three fields are not supported"
                        ),
                    },
                    Fields::Unit => quote! {
                        #name::#id => write!(f, "{}\n", stringify!(#id)),
                    },
                    _ => panic!("named fields are not supported"),
                };

                impls.push(name);
            }

            let result = quote! {
                        impl std::fmt::Display for #name {
                        #[allow(non_camel_case_types)]
                        #[allow(unused_variables)]
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            #[allow(non_snake_case)]
                             match &*self {
                                 #(#impls)*
                         }
                    }
                }
            };

            TokenStream::from(result)
        }
        _ => panic!("Macro only for enums"),
    }
}
