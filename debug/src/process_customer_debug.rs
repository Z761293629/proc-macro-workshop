use super::CustomDebugInfo;
use darling::ast;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Generics, Meta, MetaNameValue};

fn extrat_debug_attribute_value<'a>(
    attrs: &mut impl Iterator<Item = &'a syn::Attribute>,
) -> Option<&'a syn::Expr> {
    attrs.find_map(|attr| match &attr.meta {
        Meta::NameValue(MetaNameValue { path, value, .. }) if path.is_ident("debug") => Some(value),
        _ => None,
    })
}

pub(crate) fn process(info: CustomDebugInfo) -> TokenStream {
    let CustomDebugInfo {
        ident: name,
        data: ast::Data::Struct(fields),
        generics,
    } = info
    else {
        panic!("only support struct!");
    };

    let fields_chain = fields.iter().map(|field| {
        let field_name = &field.ident;
        let debug = extrat_debug_attribute_value(&mut field.attrs.iter());
        if let Some(debug) = debug {
            quote! {.field(stringify!(#field_name), &format_args!(#debug,&self.#field_name))}
        } else {
            quote! {.field(stringify!(#field_name), &self.#field_name)}
        }
    });

    let Generics { params, .. } = &generics;

    let bound = if params.is_empty() {
        quote! {}
    } else {
        let bound = params.iter().map(|generic_param| {
            quote! {
                #generic_param : std::fmt::Debug,
            }
        });

        quote! { where #(#bound)*}
    };

    quote! {
        impl #generics std::fmt::Debug for #name #generics #bound {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(#name))
                    #(#fields_chain)*
                    .finish()
            }
        }
    }
}
