use syn::{
    Fields,
    Path, Type, TypePath};


use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput};


fn get_optional_fields(input: &DeriveInput) -> Vec<String> {
    let data = match &input.data {
        Data::Struct(x) => Some(x),
        _ => panic!("only non-empty struct type can be derived for"),
    };

    let fields = match &data.unwrap().fields {
        Fields::Named(x) => Some(x),
        _ => panic!("can't use Unnamed or Unit fields for struct."),
    };

    let mut optional_fields = vec![];

    for f in fields.unwrap().named.iter() {
        match &f.ty {
            Type::Path(
                TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        // Punctuated<PathSegment, Colon2>
                        segments,
                    },
                }
            ) => if segments[0].ident.clone().to_string() == "Option" {
                optional_fields.push(f.ident.clone().unwrap().to_string());
            },
            _ => ()
        };
    };
    optional_fields
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // println!("{:?}", input);

    let opt_fields = get_optional_fields(&input);

    let executable_is_req = !opt_fields.contains(&"executable".to_string());
    let args_is_req = !opt_fields.contains(&"args".to_string());
    let env_is_req = !opt_fields.contains(&"env".to_string());
    let current_dir_is_req = !opt_fields.contains(&"current_dir".to_string());

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
                if self.executable.is_none() && #executable_is_req {
                    return Err(Box::new(BuilderError::from("executable is None".to_string())));
                }

                if self.args.is_none() && #args_is_req {
                    return Err(Box::new(BuilderError::from("args is None".to_string())));
                }

                if self.env.is_none() && #env_is_req {
                    return Err(Box::new(BuilderError::from("env is None".to_string())));
                }

                if self.current_dir.is_none() && #current_dir_is_req {
                    return Err(Box::new(BuilderError::from("current_dir is None".to_string())));
                }


                // let executable = self.executable.clone();
                // let args = self.args.clone();
                // let env = self.env.clone();
                let current_dir = self.current_dir.clone();

                if #current_dir_is_req {
                    let current_dir = self.current_dir.clone().unwrap(); };

                // println!("Executable: {}", executable);
                // println!("Args: {}", args);
                // println!("Env: {}", env);
                // println!("CurrentDir: {}", current_dir);

                Ok(
                    Command {
                        executable: executable.clone().unwrap(),
                        args: args.clone().unwrap(),
                        env: env.clone().unwrap(),
                        current_dir: current_dir
                })
            }
        }
    };

    TokenStream::from(expanded)
    // TokenStream::new()
}

