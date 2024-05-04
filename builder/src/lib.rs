use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput,
    parse_macro_input,
};

#[proc_macro_derive(Builder)]
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
                                ) =>
                                    match segments.first() {
                                        Some(syn::PathSegment {
                                            ident,
                                            ..
                                        }) if segments.len() == 1 =>
                                            if ident == "Option" {
                                                (field_ident.clone(), true)
                                            } else {
                                                (field_ident.clone(), false)
                                            },
                                        _ => unimplemented!()
                                    },
                                _ => unimplemented!(),
                            }
                        ).collect::<Vec<_>>(),
                    _ => unimplemented!(),
                }
            _ => unimplemented!(),
        };
    let option_fields = fields.iter().filter(|(_, is_option)| *is_option).map(|(field, _)| field);
    let non_option_fields = fields.iter().filter(|(_, is_option)| !*is_option).map(|(field, _)| field);
    quote! {
        pub struct CommandBuilder {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }

        impl Command {
            pub fn builder() -> CommandBuilder {
                CommandBuilder {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }

        impl CommandBuilder {
            pub fn executable(&mut self, executable: String) -> &mut Self {
                self.executable = Some(executable);
                self
            }

            pub fn args(&mut self, args: Vec<String>) -> &mut Self {
                self.args = Some(args);
                self
            }

            pub fn env(&mut self, env: Vec<String>) -> &mut Self {
                self.env = Some(env);
                self
            }

            pub fn current_dir(&mut self, current_dir: String) -> &mut Self {
                self.current_dir = Some(current_dir);
                self
            }

            pub fn build(&self) -> Result<Command, Box<dyn Error>> {
                Ok(Command {
                    #(
                        #option_fields: self.#option_fields.clone(),
                    )*
                    #(
                        #non_option_fields: self.#non_option_fields.clone().ok_or(format!("{} is required", stringify!(#non_option_fields)))?,
                    )*
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
