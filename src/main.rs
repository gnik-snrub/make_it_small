use cli::run_command;

pub mod cli;
pub mod constants;
pub mod headers;
pub mod io;
pub mod tests;
pub mod huffman;
pub mod flags;
pub mod checksum;
pub mod archive;
pub mod crypto;

fn main() {
    let _ = run_command();
}
