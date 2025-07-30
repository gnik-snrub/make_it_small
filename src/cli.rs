use std::path::Path;

use clap::{Parser, Subcommand};

use crate::{huffman::{decoder::decode, encoder::encode}, io::{read_file, write_file}};

#[derive(Parser, Debug)]
#[command(name = "Make It Small")]
#[command(about = "File compression and decompression application", long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>
}

#[derive(Subcommand, Debug)]
enum Command {
    Compress {
        name_in: String,
        name_out: Option<String>
    },
    Decompress {
        name_in: String,
        name_out: Option<String>
    }
}

pub fn run_command() {
    let tokens = Cli::parse();
    match tokens {
        Cli { command: Some(Command::Compress { name_in, name_out }) } => {
            let file = read_file(&name_in);
            let name = Path::new(&name_in).file_name().unwrap().to_str().unwrap();
            let small_file = encode(&file, name);
            let output_path = match name_out {
                Some(name) => format!("{name}.small"),
                None => format!("{}.small", name_in),
            };
            write_file(small_file, &output_path);
        },
        Cli { command: Some(Command::Decompress { name_in, name_out}) } => {
            let file = read_file(&name_in);
            let (decompressed, original_name) = decode(file);
            let output_path = match name_out {
                Some(name) => format!("{name}"),
                None => format!("{original_name}"),
            };
            write_file(decompressed, &format!("{}", output_path));
        },
        Cli { command: None } => {

        },
    }
}
