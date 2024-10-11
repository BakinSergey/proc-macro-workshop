use proc_macro::TokenStream;

#[proc_macro]
pub fn show_token_stream(args: TokenStream) -> TokenStream {
    println!("Поток токенов:");
    for tt in args {
        println!(" {tt:?}");
    }
    // возвращаем пустой поток токенов
    TokenStream::new()
}