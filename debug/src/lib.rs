use darling::{ast, FromDeriveInput, FromField};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Generics, Ident, Type};
mod process_customer_debug;
#[derive(Debug, FromDeriveInput)]
struct CustomDebugInfo {
    ident: Ident,
    data: ast::Data<(), FieldInfo>,
    generics: Generics,
}

#[derive(Debug, FromField)]
#[allow(unused)]
#[darling(forward_attrs(debug))]
struct FieldInfo {
    ident: Option<Ident>,
    ty: Type,
    attrs: Vec<syn::Attribute>,
}

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match CustomDebugInfo::from_derive_input(&ast) {
        Ok(info) => process_customer_debug::process(info).into(),
        Err(e) => e.write_errors().into(),
    }
}
