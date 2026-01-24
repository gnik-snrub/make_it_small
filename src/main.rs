use cli::run_command;

pub mod archive;
pub mod checksum;
pub mod cli;
pub mod constants;
pub mod crypto;
pub mod errors;
pub mod flags;
pub mod headers;
pub mod huffman;
pub mod io;
pub mod progress;
pub mod tests;
pub mod ux;

fn main() {
    let _ = run_command();
}
