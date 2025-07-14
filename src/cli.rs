use clap::{Parser, Subcommand};

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
        name_out: String,
    }
}

pub fn run_command() {
    let tokens = Cli::parse();
    match tokens {
        Cli { command: Some(Command::Compress { name_in, name_out }) } => {
            println!("{} => {}", name_in, name_out);
        },
        Cli { command: Some(Command::Decompress { name_in, name_out }) } => {
            println!("{} => {}", name_in, name_out);
        },
        Cli { command: None } => {

        },
    }
}
