use proc_macro::TokenStream;
use syn::{DeriveInput, Field, parse_macro_input};

mod common;
mod field_name;
mod getter;
mod mut_getter;
mod setter;

use common::parse_named_fields;

#[proc_macro_derive(FieldName)]
pub fn derive_field_name(input: TokenStream) -> TokenStream {
    derive_named_struct_macro(input, "FieldName", field_name::expand)
}

#[proc_macro_derive(Getter)]
pub fn derive_getter(input: TokenStream) -> TokenStream {
    derive_named_struct_macro(input, "Getter", getter::expand)
}

#[proc_macro_derive(MutGetter)]
pub fn derive_mut_getter(input: TokenStream) -> TokenStream {
    derive_named_struct_macro(input, "MutGetter", mut_getter::expand)
}

#[proc_macro_derive(Setter)]
pub fn derive_setter(input: TokenStream) -> TokenStream {
    derive_named_struct_macro(input, "Setter", setter::expand)
}

fn derive_named_struct_macro(
    input: TokenStream,
    macro_name: &'static str,
    expand: fn(syn::Ident, syn::Generics, Vec<Field>) -> proc_macro2::TokenStream,
) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match parse_named_fields(input, macro_name) {
        Ok((ty, generics, fields)) => expand(ty, generics, fields).into(),
        Err(err) => err.to_compile_error().into(),
    }
}
