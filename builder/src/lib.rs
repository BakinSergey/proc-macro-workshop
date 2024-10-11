mod utils;

use utils::extract_type_from_option;

use syn::{Fields, Path, Type, TypePath};

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use std::collections::HashMap;
use syn::{parse_macro_input, Data, DeriveInput};


fn get_fields_info(input: &DeriveInput) -> (HashMap<String, bool>, HashMap<String, Type>) {
    let data = match &input.data {
        Data::Struct(x) => Some(x),
        _ => panic!("only non-empty struct type can be derived for"),
    };

    let fields = match &data.unwrap().fields {
        Fields::Named(x) => Some(x),
        _ => panic!("can't use Unnamed or Unit fields for struct."),
    };

    let mut fields_kind = HashMap::new();
    let mut fields_type = HashMap::new();

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
            ) => {
                // Обязательность поля
                let is_mandatory = segments[0].ident.clone().to_string() != "Option";

                // имя поля
                let f_name = f.ident.clone().unwrap().to_string();

                //тип поля
                if is_mandatory {
                    fields_type.insert(f_name.clone(), f.ty.clone());
                } else {
                    let t = extract_type_from_option(&f.ty).unwrap_or(&f.ty);
                    fields_type.insert(f_name.clone(), t.clone());
                }

                fields_kind.insert(f_name, is_mandatory);
            }
            _ => ()
        };
    };

    (fields_kind, fields_type)
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

fn gen_impl_builder_code(fields_req: &HashMap<String, bool>, fields_tip: &HashMap<String, Type>) -> TokenStream2 {
    let struct_fields_setters: Vec<_> = fields_req.iter().map(|(field_name, is_mandatory)| {
        match is_mandatory {
            true => {
                let field_ident = format_ident!("{}", field_name);
                let field_type = &fields_tip[field_name];

                quote::quote! {
                fn #field_ident(&mut self, val: #field_type) -> &mut Self {
                    self.#field_ident = val;
                    self
                }
            }
            }
            false => {
                let field_ident = format_ident!("{}", field_name);
                let field_type = &fields_tip[field_name];

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
    let (fields_kind, field_type) = get_fields_info(&input);

    // генерация билд-структуры по прототипу(теже поля и типы полей) вызывающей структуры
    let builder_struct_code = gen_builder_struct_code(&input);

    // Генерация кода функций-сеттеров в зависимости от опциональности полей
    let impl_command_builder_code = gen_impl_builder_code(&fields_kind, &field_type);

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

