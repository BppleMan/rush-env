use syn::{Data, DeriveInput, Field, Fields};

pub fn parse_named_fields(input: DeriveInput, macro_name: &'static str) -> syn::Result<(syn::Ident, syn::Generics, Vec<Field>)> {
    let ty = input.ident;
    let generics = input.generics;
    match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => Ok((ty, generics, fields.named.into_iter().collect())),
            _ => Err(syn::Error::new_spanned(ty, format!("{macro_name} 只能用于具名字段结构体"))),
        },
        _ => Err(syn::Error::new_spanned(ty, format!("{macro_name} 目前只支持结构体"))),
    }
}

pub fn normalized_field_name(ident: &syn::Ident) -> String {
    let s = ident.to_string();
    s.strip_prefix("r#").unwrap_or(&s).to_string()
}
