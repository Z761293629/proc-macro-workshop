use crate::FieldInfo;

use super::DeriveInputInfo;
use darling::ast::{self, Fields};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{AngleBracketedGenericArguments, GenericArgument, Path, PathArguments, Type, TypePath};

enum InnerType {
    OptionType(Type),
    VecType(Type),
    Primitive,
}

impl From<&Type> for InnerType {
    fn from(value: &Type) -> Self {
        match value {
            Type::Path(TypePath {
                path: Path { ref segments, .. },
                ..
            }) => {
                let ident = &segments[0].ident;
                if ident == "Option" {
                    InnerType::OptionType(extract_inner_type(&segments[0].arguments).unwrap())
                } else if ident == "Vec" {
                    InnerType::VecType(extract_inner_type(&segments[0].arguments).unwrap())
                } else {
                    InnerType::Primitive
                }
            }
            _ => InnerType::Primitive,
        }
    }
}

fn extract_inner_type(argument: &PathArguments) -> Option<Type> {
    match argument {
        PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => {
            match args.first().unwrap() {
                GenericArgument::Type(ty) => Some(ty.clone()),
                _ => None,
            }
        }
        _ => None,
    }
}

fn handle_builder_fields(fields: &Fields<FieldInfo>) -> impl Iterator<Item = TokenStream> + '_ {
    fields.iter().map(|field| {
        let field_name = &field.ident;
        let inner_type = InnerType::from(&field.ty);
        let ty = match inner_type {
            InnerType::OptionType(ty) => ty,
            _ => field.ty.clone(),
        };
        quote! { #field_name : ::std::option::Option<#ty>,}
    })
}

fn handle_builder_build_field(
    fields: &Fields<FieldInfo>,
) -> impl Iterator<Item = TokenStream> + '_ {
    fields.iter().map(|field| {
        let field_name = &field.ident;
        let inner_type = InnerType::from(&field.ty);

        match inner_type {
            InnerType::OptionType(_) => quote! {#field_name: self.#field_name.take(),},
            InnerType::VecType(_) => {
                quote! {#field_name: self.#field_name.take().unwrap_or(Vec::new()),}
            }
            InnerType::Primitive => quote! {
                #field_name: self.#field_name.take().ok_or(format!("{} is not set", stringify!(#field_name)))?,
            },
        }
    })
}

fn handle_builder_fields_init(
    fields: &Fields<FieldInfo>,
) -> impl Iterator<Item = TokenStream> + '_ {
    fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! { #field_name : ::std::option::Option::None,}
    })
}
fn handle_builder_setter_method(
    fields: &Fields<FieldInfo>,
) -> impl Iterator<Item = TokenStream> + '_ {
    fields.iter().map(|field| {
        let field_name = &field.ident;
        let ty = &field.ty;
        let inner_type = InnerType::from(ty);

        match inner_type {
            InnerType::OptionType(inner_type) => {
                quote! {
                    fn #field_name(&mut self,#field_name:#inner_type)-> &mut Self{
                        self.#field_name = ::std::option::Option::Some(#field_name);
                        self
                    }
                }
            }
            InnerType::VecType(inner_type) if field.each.is_some() => {
                let each_name = field.each.clone().unwrap();
                let each = format_ident!("{}", each_name);
                let each = quote! {
                    fn #each(&mut self,#each:#inner_type)-> &mut Self{
                        if let Some(ref mut values) = self.#field_name {
                            values.push(#each);
                        } else {
                            self.#field_name = ::std::option::Option::Some(vec![#each]);
                        }
                        self
                    }
                };

                if field_name.clone().unwrap() == each_name {
                    return each;
                } else {
                    quote! {
                        #each

                        fn #field_name(&mut self,#field_name:#ty)-> &mut Self{
                            self.#field_name = ::std::option::Option::Some(#field_name);
                            self
                        }
                    }
                }
            }
            _ => {
                quote! {
                    fn #field_name(&mut self,#field_name:#ty)-> &mut Self{
                        self.#field_name = ::std::option::Option::Some(#field_name);
                        self
                    }
                }
            }
        }
    })
}

pub(crate) fn process(input_info: DeriveInputInfo) -> TokenStream {
    let DeriveInputInfo {
        ident: name,
        data: ast::Data::Struct(fields),
    } = input_info
    else {
        panic!("only support struct");
    };
    let builder = format_ident!("{}Builder", name);

    let builder_fields = handle_builder_fields(&fields);

    let builder_fields_init = handle_builder_fields_init(&fields);

    let methods = handle_builder_setter_method(&fields);

    let extractors = handle_builder_build_field(&fields);
    quote! {
        struct #builder{
            #(#builder_fields)*
        }

        impl #builder{
            #(#methods)*
        }

        impl #builder{
            pub fn build(&mut self) -> ::std::result::Result<#name,::std::boxed::Box<dyn std::error::Error>>{
                Ok(#name {
                    #(
                        #extractors
                    )*
                })
            }
        }

        impl #name {
            fn builder() -> #builder{
                #builder{
                    #(#builder_fields_init)*
                }
            }
        }

    }
}
