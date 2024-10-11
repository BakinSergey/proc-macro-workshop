use derive_derinput::DerInput;

#[derive(DerInput)]
pub struct Command {
    executable: String,
    // args: Vec<String>,
    // env: Vec<String>,
    current_dir: Option<String>,
}

fn main() {}