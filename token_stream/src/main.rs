use token_stream::show_token_stream;

fn main() {
    show_token_stream!(
        pub struct Command {
        executable: String,
        // args: Vec<String>,
        // env: Vec<String>,
        current_dir: Option<String>,
    }
    );
}
