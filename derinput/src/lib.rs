use syn::{
    Fields,
    Path, Type, TypePath};


use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(DerInput)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    println!("==> attrs:");
    for att in &input.attrs {
        println!("  {att:?}")
    }

    println!("==> vis:");
    println!("  {:?}", &input.vis);

    println!("==> ident:");
    println!("  {:?}", &input.ident);

    println!("==> generics:");
    println!("  {:?}", &input.generics);

    println!("==> data:");
    println!(" {:?}", &input.data);

    TokenStream::new()
}

