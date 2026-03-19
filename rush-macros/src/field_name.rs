use quote::{format_ident, quote};
use syn::Field;

use crate::common::normalized_field_name;

pub fn expand(ty: syn::Ident, generics: syn::Generics, fields: Vec<Field>) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let methods = fields.iter().map(|field| {
        let ident = field.ident.as_ref().expect("named field");
        let field_name = normalized_field_name(ident);
        let fn_ident = format_ident!("field_{}", field_name);
        quote! {
            pub const fn #fn_ident() -> &'static str {
                #field_name
            }
        }
    });
    quote! {
        impl #impl_generics #ty #ty_generics #where_clause {
            #(#methods)*
        }
    }
}
