// Test that it is possible to use the macro twice in the same module.

use derive_builder::Builder;

#[derive(Builder)]
pub struct Command {
    executable: String,
}
#[derive(Builder)]
pub struct Command2 {
    executable: String,
}

fn main() {}
