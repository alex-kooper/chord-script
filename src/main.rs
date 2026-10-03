mod cli;

use std::process::ExitCode;

fn main() -> ExitCode {
    cli::run(&cli::Args::parse_or_exit())
}
