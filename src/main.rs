use cli::run_command;

pub mod cli;
pub mod constants;
pub mod headers;
pub mod io;
pub mod tests;

fn main() {
    run_command();
}
