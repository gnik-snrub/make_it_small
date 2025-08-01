use cli::run_command;

pub mod cli;
pub mod constants;
pub mod headers;
pub mod io;
pub mod tests;
pub mod huffman;
pub mod flags;

fn main() {
    run_command();
}
