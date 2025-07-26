use std::path::Path;

use clap::{Parser, Subcommand};

use crate::{headers::{write_header, Headers}, io::{read_file, write_file}};

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
        name_out: String,
    },
    Decompress {
        name_in: String,
    }
}

pub fn run_command() {
    let tokens = Cli::parse();
    match tokens {
        Cli { command: Some(Command::Compress { name_in, mut name_out }) } => {
            let file = read_file(&name_in);
            let name = Path::new(&name_in).file_name().unwrap().to_str().unwrap();
            let headers = write_header(&file, name);
            let mut small_file = headers.to_bytes();
            small_file.extend_from_slice(&file);
            name_out.push_str(".small");
            write_file(small_file, &name_out);
        },
        Cli { command: Some(Command::Decompress { name_in }) } => {
            let file = read_file(&name_in);
            let (headers, header_length) = match Headers::from_bytes(&file) {
                Ok(header_and_length) => {
                    header_and_length
                }
                Err(e) => {
                    panic!("Error deserializing headers: {e}");
                }
            };
            write_file(file[header_length..].to_vec(), &headers.original_file_name);
        },
        Cli { command: None } => {

        },
    }
}
