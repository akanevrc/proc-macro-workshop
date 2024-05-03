use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput,
    parse_macro_input,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = parse_macro_input!(input as DeriveInput);
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
                    executable: self.executable.clone().ok_or("executable is required")?,
                    args: self.args.clone().ok_or("args is required")?,
                    env: self.env.clone().ok_or("env is required")?,
                    current_dir: self.current_dir.clone().ok_or("current_dir is required")?,
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
