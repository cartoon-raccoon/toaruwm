//! Procedural macros for use in Toaru.

use proc_macro2::{TokenStream, Span};

use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, FieldsNamed, Meta};
use quote::quote;

/// Derives the `ConfigSection` trait.
/// 
/// ## Example
/// 
/// ```ignore
/// #[derive(ConfigSection)]
/// pub struct SomeSection {
///     #[key] // gets exposed as a key on the configuration.
///     some_key: String,
///     #[key]
///     other_key: bool,
///     
///     #[subsection] // gets exposed as a subsection on the configuration.
///     section: SomeSubSection,
/// 
///     other_field: u32 // fields with no attribute tags do not get exposed.
/// }
/// ```
#[proc_macro_derive(ConfigSection, attributes(key, subsection))]
pub fn configsection_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand_configsection(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_configsection(input: DeriveInput) -> syn::Result<TokenStream> {
    let Data::Struct(datastruct) = input.data else {
        return Err(syn::Error::new(
            Span::call_site(), "ConfigSection can only be derived on structs"))
    };

    let Fields::Named(fields) = datastruct.fields else {
        return Err(syn::Error::new(
            Span::call_site(), "ConfigSection cannot be derived on structs with unnamed fields"))
    };

    let keys = parse_fields(&fields, "key");

    let subsections = parse_fields(&fields, "subsection");

    let name = input.ident;
    let generics = input.generics;

    let tokens = quote! {
        impl #generics ConfigSection for #name #generics {
            fn get_key(&self, name: &str) -> Option<&dyn std::any::Any> {
                #keys
            }

            fn subsection(&self, name: &str) -> Option<&dyn ConfigSection> {
                #subsections
            }
        }
    };

    Ok(tokens)
}

fn parse_fields(fields: &FieldsNamed, helper: &str) -> TokenStream {
    let FieldsNamed { named, .. } = fields;

    if named.is_empty() {
        return quote! { None }
    }

    let mut arms = TokenStream::new();

    for field in named {
        if field_has_helper(&field, helper) {
            let ident = field.ident.as_ref().unwrap();
            let identstr = ident.to_string();
            arms.extend(quote! { #identstr => Some(&self.#ident), });
        }
    }

    quote! {
        match name {
            #arms
            _ => None
        }
    }
}

/// Looks for `#[key]` on a given Field.
fn field_has_helper(field: &Field, helper: &str) -> bool {
    for attr in field.attrs.iter() {
        if let Meta::Path(p) = &attr.meta {
            if p.segments.len() == 1 {
                let ident = &p.segments.first().unwrap().ident;

                if ident == helper {
                    return true
                }
            }
        }
    }

    false
}