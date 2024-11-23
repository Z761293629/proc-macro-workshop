use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, AngleBracketedGenericArguments, Data, DataStruct, DeriveInput, Fields,
    FieldsNamed, GenericArgument, Path, PathArguments, Type, TypePath,
};

fn extract_option_inner_type(ty: &Type) -> Option<Type> {
    match ty {
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) if segments[0].ident == "Option" => match &segments[0].arguments {
            PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => {
                match args.first().unwrap() {
                    GenericArgument::Type(ty) => Some(ty.clone()),
                    _ => None,
                }
            }
            _ => None,
        },
        _ => None,
    }
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = ast.ident;
    let builder = format_ident!("{}Builder", name);

    let origin_fields = match ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => named,
        _ => unreachable!(),
    };

    let initialization = origin_fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! { #field_name : None ,}
    });

    let fields = origin_fields.iter().map(|field| {
        let field_name = &field.ident;
        let ty = &field.ty;

        match extract_option_inner_type(ty) {
            Some(ty) => quote! { #field_name : Option<#ty>,},
            None => quote! { #field_name : Option<#ty>,},
        }
    });

    let setter_methods = origin_fields.iter().map(|field| {
        let field_name = &field.ident;
        let ty = &field.ty;

        let real_type = match extract_option_inner_type(ty) {
            Some(ty) => ty,
            None => ty.clone(),
        };
        quote! {
            fn #field_name(&mut self,#field_name: #real_type) -> &mut Self{
                self.#field_name = Some(#field_name);
                self
            }
        }
    });

    let checkers = origin_fields.iter().map(|field| {
        let field_name = &field.ident;
        let ty = &field.ty;
        let error_message = format!(
            "{} field is missing in {}",
            field_name.clone().unwrap(),
            name
        );
        match extract_option_inner_type(ty) {
            Some(_) => quote! {},
            None => quote! {
                if self.#field_name.is_none(){
                return Err(#error_message.into());
            }},
        }
    });

    let extractors = origin_fields.iter().clone().map(|field| {
        let field_name = &field.ident;
        let ty = &field.ty;

        match extract_option_inner_type(ty) {
            Some(_) => quote! { #field_name : self.#field_name.clone(), },
            None => quote! { #field_name : self.#field_name.clone().unwrap(), },
        }
    });

    let token = quote! {
        use std::error::Error;

        pub struct #builder {
            #(
                #fields
            )*
        }

        impl #builder {
            #(
                #setter_methods
            )*
        }

        impl #builder {
            pub fn build(&mut self) -> Result<#name,Box<dyn Error>>{
                #(
                    #checkers
                )*

                Ok(
                    #name {
                    #(
                        #extractors
                    )*
                }
            )

            }
        }

        impl #name {
            pub fn builder() -> #builder {
                #builder {
                    #(#initialization)*
                }
            }
        }

    };
    token.into()
}
