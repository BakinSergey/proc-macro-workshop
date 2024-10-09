use proc_macro::TokenStream;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = input;
    let expanded = quote::quote! {

        use std::result::Result;
        use std::error::Error;
        use std::fmt::{Display, Formatter, Result as FmtResult};

        pub struct CommandBuilder {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }

        #[derive(Debug)]
        pub struct BuilderError {
            pub msg: String
        }

        impl Display for BuilderError {
            fn fmt(&self, f: &mut Formatter) -> FmtResult {
                write!(f, "{}", self.msg)
            }
        }

        impl Error for BuilderError {
            fn description(&self) -> &str {
                &self.msg
            }
        }

        impl From<String> for BuilderError {
            fn from(err: String) -> BuilderError {
                BuilderError { msg: err }
            }
        }

        impl CommandBuilder {
            fn executable(&mut self, executable: String) -> &mut Self {
              self.executable = Some(executable);
                self
            }

            fn args(&mut self, args: Vec<String>) -> &mut Self {
                self.args = Some(args);
                self
            }

            fn env(&mut self, env: Vec<String>) -> &mut Self {
                self.env = Some(env);
                self
            }

            fn current_dir(&mut self, current_dir: String) -> &mut Self {
                self.current_dir = Some(current_dir);
                self
            }

            fn build(&mut self) -> Result<Command, Box<dyn Error>> {
                if let None = &self.executable {
                    return Err(Box::new(BuilderError::from("executable is None".to_string())));
                }

                if let None = &self.args {
                    return Err(Box::new(BuilderError::from("args is None".to_string())));
                }

                if let None = &self.env {
                    return Err(Box::new(BuilderError::from("env is None".to_string())));
                }

                if let None = &self.current_dir {
                    return Err(Box::new(BuilderError::from("current_dir is None".to_string())));
                }

                Ok(
                    Command {
                        executable: self.executable.clone().unwrap(),
                        args: self.args.clone().unwrap(),
                        env: self.env.clone().unwrap(),
                        current_dir: self.current_dir.clone().unwrap()
                })

            }

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
    };
    TokenStream::from(expanded)
}
