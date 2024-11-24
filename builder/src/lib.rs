use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Ident, Type};
mod process_builder;
#[derive(Debug, FromDeriveInput)]
struct DeriveInputInfo {
    ident: Ident,
    data: darling::ast::Data<(), FieldInfo>,
}

#[derive(Debug, FromField)]
#[darling(attributes(builder))]
struct FieldInfo {
    ident: Option<Ident>,
    ty: Type,
    each: Option<String>,
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match DeriveInputInfo::from_derive_input(&ast) {
        Ok(input_info) => process_builder::process(input_info).into(),
        Err(e) => e.write_errors().into(),
    }
}
