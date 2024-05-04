use proc_macro::TokenStream;
use quote::{
    format_ident,
    quote,
};
use syn::{
    parse_macro_input,
    DeriveInput,
};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let fields =
        match &input.data {
            syn::Data::Struct(data) =>
                match &data.fields {
                    syn::Fields::Named(fields) =>
                        fields.named.iter().map(|field|
                            match (&field.ident, &field.ty) {
                                (
                                    Some(field_ident),
                                    syn::Type::Path(
                                        syn::TypePath {
                                            qself: None,
                                            path: syn::Path {
                                                segments,
                                                ..
                                            },
                                        },
                                    ),
                                ) if segments.len() == 1 =>
                                    match &segments[0] {
                                        syn::PathSegment {
                                            ident,
                                            ..
                                        } => {
                                            let is_option = ident == "Option";
                                            let builder_ty =
                                                if is_option {
                                                    match &segments[0].arguments {
                                                        syn::PathArguments::AngleBracketed(
                                                            syn::AngleBracketedGenericArguments {
                                                                args,
                                                                ..
                                                            },
                                                        ) => {
                                                            match args.first() {
                                                                Some(syn::GenericArgument::Type(ty)) => ty.clone(),
                                                                _ => unimplemented!(),
                                                            }
                                                        },
                                                        _ => unimplemented!(),
                                                    }
                                                }
                                                else {
                                                    field.ty.clone()
                                                };
                                            let each_attr =
                                                field.attrs.iter()
                                                .find(|attr| attr.path().is_ident("builder"))
                                                .and_then(|attr|
                                                    match attr.parse_args() {
                                                        Ok(syn::Meta::NameValue(syn::MetaNameValue {
                                                            path: syn::Path {
                                                                segments,
                                                                ..
                                                            },
                                                            value: syn::Expr::Lit(syn::ExprLit {
                                                                lit: syn::Lit::Str(lit_str),
                                                                ..
                                                            }),
                                                            ..
                                                        })) if segments.len() == 1 => {
                                                            match &segments[0] {
                                                                syn::PathSegment {
                                                                    ident,
                                                                    ..
                                                                } if ident == "each" => {
                                                                    Some(lit_str)
                                                                },
                                                                _ => unimplemented!(),
                                                            }},
                                                        _ => None,
                                                    }
                                                );
                                            (field_ident.clone(), is_option, builder_ty, each_attr)
                                        },
                                    },
                                _ => unimplemented!(),
                            }
                        ).collect::<Vec<_>>(),
                    _ => unimplemented!(),
                }
            _ => unimplemented!(),
        };
    let command_field_names = fields.iter().map(|(ident, _, _, _)| ident);
    let builder_field_names = command_field_names.clone();
    let builder_field_types =
        fields.iter()
        .map(|(_, _, builder_ty, each_attr)|
            match each_attr {
                Some(_) => quote! { #builder_ty },
                None => quote! { Option<#builder_ty> },
            }
        );
    let command_field_values =
        fields.iter()
        .map(|(_, _, _, each_attr)|
            match each_attr {
                Some(_) => quote! { Vec::new() },
                None => quote! { None },
            }
        );
    let builder_method_defs =
        fields.iter()
        .map(|(ident, _, builder_ty, each_attr)|
            match each_attr {
                Some(lit) => {
                    let method_ident = format_ident!("{}", lit.value());
                    quote! {
                        pub fn #method_ident(&mut self, value: String) -> &mut Self {
                            self.#ident.push(value);
                            self
                        }
                    }
                },
                None =>
                    quote! {
                        pub fn #ident(&mut self, value: #builder_ty) -> &mut Self {
                            self.#ident = Some(value);
                            self
                        }
                    },
            }
        );
    let command_field_inits =
        fields.iter()
        .map(|(ident, is_option, _, each_attr)|
            match (is_option, each_attr) {
                (true, Some(_)) =>
                    quote! {
                        #ident: Some(self.#ident.clone()),
                    },
                (true, None) =>
                    quote! {
                        #ident: self.#ident.clone(),
                    },
                (false, Some(_)) =>
                    quote! {
                        #ident: self.#ident.clone(),
                    },
                (false, None) =>
                    quote! {
                        #ident: self.#ident.clone().ok_or(format!("{} is required", stringify!(#ident)))?,
                    },
            }
        );
    quote! {
        pub struct CommandBuilder {
            #(#builder_field_names: #builder_field_types,)*
        }

        impl Command {
            pub fn builder() -> CommandBuilder {
                CommandBuilder {
                    #(#command_field_names: #command_field_values,)*
                }
            }
        }

        impl CommandBuilder {
            #(#builder_method_defs)*

            pub fn build(&self) -> Result<Command, Box<dyn Error>> {
                Ok(Command {
                    #(#command_field_inits)*
                })
            }
        }

        trait Error: Send + Sync + std::fmt::Debug + 'static {}

        impl Error for &'static str {}
        impl Error for String {}

        impl From<&'static str> for Box<dyn Error> {
            fn from(err: &'static str) -> Box<dyn Error> {
                Box::new(err)
            }
        }

        impl From<String> for Box<dyn Error> {
            fn from(err: String) -> Box<dyn Error> {
                Box::new(err)
            }
        }
    }.into()
}
