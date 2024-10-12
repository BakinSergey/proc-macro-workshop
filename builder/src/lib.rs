mod utils;

use utils::extract_type_from_option;

use syn::{Fields, Path, Type, TypePath};

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use std::collections::HashMap;
use syn::Ident;
use syn::{parse_macro_input, Data, DeriveInput};
use syn::FieldsNamed;

struct FieldInfo {
    is_mandatory: bool,
    type_: Type,
}

fn extract_fields(input: &DeriveInput) -> &FieldsNamed {
    match &input.data {
        Data::Struct(ref s) => match &s.fields {
            Fields::Named(ref fields) => fields,
            _ => panic!("can be derived only for structs with named fields"),
        },
        _ => panic!("can be derived only for structs"),
    }
}

fn get_fields_info(fields: &FieldsNamed) -> HashMap<Ident, FieldInfo> {
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

fn gen_builder_struct_code(structure_name: &Ident, fields: &FieldsNamed) -> TokenStream2 {
    let struct_fields = fields.named.iter().map(|field| {
        quote::quote! {
            #field
        }
    });

    let name = format_ident!("{}Builder", structure_name);

    let generated_code = quote::quote! {

    #[derive(Debug, Default)]
    struct #name {
            #(#struct_fields),*
      }
    };

    generated_code
}

fn gen_impl_builder_code(fields: &HashMap<Ident, FieldInfo>) -> TokenStream2 {
    let struct_fields_setters: Vec<_> = fields.iter().map(|(name, info)| {
        let field_value = match info.is_mandatory {
            true => quote::quote! { val },
            false => quote::quote! { Some(val) }
        };
        let field_ident = &name;
        let field_type = &info.type_;

        quote::quote! {
            fn #field_ident(&mut self, val: #field_type) -> &mut Self {
                self.#field_ident = #field_value;
                self
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

    let input_fields = extract_fields(&input);
    // данные Опциональности и Типов полей
    let fields = get_fields_info(&input_fields);

    // генерация билд-структуры по прототипу(теже поля и типы полей) вызывающей структуры
    let builder_struct_code = gen_builder_struct_code(&input.ident, &input_fields);

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

