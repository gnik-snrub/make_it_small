use cli::run_command;

pub mod archive;
#[cfg(feature = "cli")]
pub mod archive_legacy;
pub mod checksum;
#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "compression")]
pub mod compress;
pub mod constants;
pub mod crypto;
pub mod error;
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
