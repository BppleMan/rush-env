use quote::{format_ident, quote};
use syn::Field;

use crate::common::normalized_field_name;

pub fn expand(ty: syn::Ident, generics: syn::Generics, fields: Vec<Field>) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let methods = fields.iter().map(|field| {
        let ident = field.ident.as_ref().expect("named field");
        let field_ty = &field.ty;
        let fn_ident = format_ident!("get_{}", normalized_field_name(ident));
        quote! {
            pub fn #fn_ident(&self) -> &#field_ty {
                &self.#ident
            }
        }
    });
    quote! {
        impl #impl_generics #ty #ty_generics #where_clause {
            #(#methods)*
        }
    }
}
