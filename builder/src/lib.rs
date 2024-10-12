mod utils;

use utils::extract_type_from_option;

use syn::{Fields, Path, Type, TypePath};

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use std::collections::HashMap;
use syn::Ident;
use syn::{parse_macro_input, Data, DeriveInput};

struct FieldInfo {
    is_mandatory: bool,
    type_: Type,
}


fn get_fields_info(input: &DeriveInput) -> HashMap<Ident, FieldInfo> {
    let data = match &input.data {
        Data::Struct(x) => x,
        _ => panic!("only non-empty struct type can be derived for"),
    };

    let fields = match &data.fields {
        Fields::Named(x) => x,
        _ => panic!("can't use Unnamed or Unit fields for struct."),
    };

    let mut infos = HashMap::new();

    for f in fields.named.iter() {
        match f.ty {
            Type::Path(
                TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        // Punctuated<PathSegment, Colon2>
                        ref segments,
                    },
                }
            ) => {
                // Обязательность поля
                let is_mandatory = segments[0].ident != "Option";

                // имя поля
                let f_name = f.ident.clone().expect("Named field expected");

                //тип поля
                let f_type = if is_mandatory {
                    &f.ty
                } else {
                    extract_type_from_option(&f.ty).unwrap_or(&f.ty)
                };
                infos.insert(f_name.clone(), FieldInfo { is_mandatory, type_: f_type.clone() });
            }
            _ => unreachable!(),
        };
    };

    infos
}

fn gen_builder_struct_code(derive_input: &DeriveInput) -> TokenStream2 {
    let name = &derive_input.ident;

    let fields = match &derive_input.data {
        Data::Struct(ref s) => s.fields.iter().collect::<Vec<_>>(),
        _ => panic!("can be derived only for structs"),
    };

    let struct_fields: Vec<_> = fields.iter().map(|field| {
        quote::quote! {
            #field
        }
    }).collect();

    let name = format!("{}Builder", name);
    let struct_name = format_ident!("{}", name);

    let generated_code = quote::quote! {

    #[derive(Debug, Default)]
    struct #struct_name {
            #(#struct_fields),*
      }
    };

    generated_code
}

fn gen_impl_builder_code(fields: &HashMap<Ident, FieldInfo>) -> TokenStream2 {
    let struct_fields_setters: Vec<_> = fields.iter().map(|(name, info)| {
        match info.is_mandatory {
            true => {
                let field_ident = &name;
                let field_type = &info.type_;

                quote::quote! {
                fn #field_ident(&mut self, val: #field_type) -> &mut Self {
                    self.#field_ident = val;
                    self
                }
            }
            }
            false => {
                let field_ident = &name;
                let field_type = &info.type_;

                quote::quote! {
                fn #field_ident(&mut self, val: #field_type) -> &mut Self {
                    self.#field_ident = Some(val);
                    self
                }
            }
            }
        }
    }).collect();

    let generated_code = quote::quote! {

        impl CommandBuilder {
                #(#struct_fields_setters) *

                fn build(&mut self) -> Result<Command, Box<dyn Error>> {
                    Ok(
                        Command {
                            executable: self.executable.clone(),
                            args: self.args.clone(),
                            env: self.env.clone(),
                            current_dir: self.current_dir.clone()
                    })
                }
        };

    };
    generated_code
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // данные Опциональности и Типов полей
    let fields = get_fields_info(&input);

    // генерация билд-структуры по прототипу(теже поля и типы полей) вызывающей структуры
    let builder_struct_code = gen_builder_struct_code(&input);

    // Генерация кода функций-сеттеров в зависимости от опциональности полей
    let impl_command_builder_code = gen_impl_builder_code(&fields);

    let expanded = quote::quote! {

        use std::result::Result;
        use std::error::Error;
        use std::fmt::{Display, Formatter, Result as FmtResult};

        #builder_struct_code

        impl Command {
            pub fn builder() -> CommandBuilder {
                CommandBuilder::default()
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

        #impl_command_builder_code

    };

    TokenStream::from(expanded)
}

