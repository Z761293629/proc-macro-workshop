use super::CustomDebugInfo;
use darling::ast;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, GenericParam, Meta, MetaNameValue};

fn extract_debug_attribute_value<'a>(
    attrs: &mut impl Iterator<Item = &'a syn::Attribute>,
) -> Option<&'a syn::Expr> {
    attrs.find_map(|attr| match &attr.meta {
        Meta::NameValue(MetaNameValue { path, value, .. }) if path.is_ident("debug") => Some(value),
        _ => None,
    })
}

fn extract_type_name(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = ty
    {
        if let Some(syn::PathSegment { ref ident, .. }) = segments.last() {
            return Some(ident.to_string());
        }
    }
    return None;
}

fn extract_phantom_generic_type_name(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = ty
    {
        if let Some(syn::PathSegment {
            ref ident,
            ref arguments,
        }) = segments.last()
        {
            if ident == "PhantomData" {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    args,
                    ..
                }) = arguments
                {
                    if let Some(syn::GenericArgument::Type(syn::Type::Path(ref gp))) = args.first()
                    {
                        if let Some(generic_ident) = gp.path.segments.first() {
                            return Some(generic_ident.ident.to_string());
                        }
                    }
                }
            }
        }
    }
    return None;
}

pub(crate) fn process(info: CustomDebugInfo) -> TokenStream {
    let CustomDebugInfo {
        ident: name,
        data: ast::Data::Struct(fields),
        mut generics,
    } = info
    else {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "CustomeDebug only supports struct!",
        )
        .to_compile_error();
    };

    let mut phantom_types = vec![];
    let mut field_type_names = vec![];
    let fields_chain = fields.iter().map(|field| {
        let field_name = &field.ident;

        if let Some(phantom_type_name) = extract_phantom_generic_type_name(&field.ty) {
            phantom_types.push(phantom_type_name);
        }

        if let Some(field_type_name) = extract_type_name(&field.ty) {
            field_type_names.push(field_type_name);
        }

        let debug_format = extract_debug_attribute_value(&mut field.attrs.iter());
        if let Some(debug_format) = debug_format {
            quote! {.field(stringify!(#field_name), &format_args!(#debug_format,&self.#field_name))}
        } else {
            quote! {.field(stringify!(#field_name), &self.#field_name)}
        }
    }).collect::<Vec<_>>();

    println!("{:?}", phantom_types);
    println!("{:?}", field_type_names);

    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            // 该泛型参数仅在phantom字段中使用
            if phantom_types.contains(&type_param.ident.to_string())
                && !field_type_names.contains(&type_param.ident.to_string())
            {
                continue;
            }
            type_param.bounds.push(parse_quote!(std::fmt::Debug));
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics std::fmt::Debug for #name #ty_generics #where_clause  {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(#name))
                    #(#fields_chain)*
                    .finish()
            }
        }
    }
}
