use super::CustomDebugInfo;
use darling::ast;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{GenericParam, Generics, Meta, MetaNameValue};

fn extract_debug_attribute_value<'a>(
    attrs: &mut impl Iterator<Item = &'a syn::Attribute>,
) -> Option<&'a syn::Expr> {
    attrs.find_map(|attr| match &attr.meta {
        Meta::NameValue(MetaNameValue { path, value, .. }) if path.is_ident("debug") => Some(value),
        _ => None,
    })
}

fn extract_type_arguments(ty: &syn::Type) -> Vec<String> {
    match ty {
        syn::Type::Path(type_path) => type_path
            .path
            .segments
            .iter()
            .flat_map(|segment| match &segment.arguments {
                syn::PathArguments::AngleBracketed(args) => args
                    .args
                    .iter()
                    .filter_map(|arg| {
                        if let syn::GenericArgument::Type(syn::Type::Path(param_path)) = arg {
                            param_path
                                .path
                                .segments
                                .last()
                                .map(|seg| seg.ident.to_string())
                        } else {
                            None
                        }
                    })
                    .collect(),
                _ => vec![],
            })
            .collect(),

        _ => vec![],
    }
}

pub(crate) fn process(info: CustomDebugInfo) -> TokenStream {
    let CustomDebugInfo {
        ident: name,
        data: ast::Data::Struct(fields),
        generics,
    } = info
    else {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "CustomeDebug only supports struct!",
        )
        .to_compile_error();
    };

    let fields_chain = fields.iter().map(|field| {
        let field_name = &field.ident;
        let debug_format = extract_debug_attribute_value(&mut field.attrs.iter());
        if let Some(debug_format) = debug_format {
            quote! {.field(stringify!(#field_name), &format_args!(#debug_format,&self.#field_name))}
        } else {
            quote! {.field(stringify!(#field_name), &self.#field_name)}
        }
    });

    let Generics { params, .. } = &generics;

    let bound = if params.is_empty() {
        quote! {}
    } else {
        let bound = params.iter().map(|generic_param| {
            let type_param = match generic_param {
                GenericParam::Type(type_param) => type_param.ident.to_string(),
                _ => return quote! {},
            };

            let field_type = fields.iter().find_map(|field| {
                if extract_type_arguments(&field.ty).contains(&type_param) {
                    Some(&field.ty)
                } else {
                    None
                }
            });
            match field_type {
                Some(field_type) => quote! {
                    #field_type : std::fmt::Debug,
                },
                None => quote! {
                    #generic_param : std::fmt::Debug,
                },
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
