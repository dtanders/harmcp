mod cli;
mod commands;
mod error;
mod filter;
mod har;
mod input;
mod output;

use clap::Parser;
use cli::{Cli, Command};
use output::DetailSection;

fn main() {
    let cli = Cli::parse();
    let result = match &cli.command {
        Command::List(args) => commands::list::run(&cli.file, args, &cli.format),
        Command::Headers { index } => {
            commands::detail::run(&cli.file, *index, DetailSection::Headers, &cli.format)
        }
        Command::Body { index } => {
            commands::detail::run(&cli.file, *index, DetailSection::Body, &cli.format)
        }
        Command::Timings { index } => {
            commands::detail::run(&cli.file, *index, DetailSection::Timings, &cli.format)
        }
        Command::Stack { index } => {
            commands::detail::run(&cli.file, *index, DetailSection::Stack, &cli.format)
        }
        Command::All { index } => {
            commands::detail::run(&cli.file, *index, DetailSection::All, &cli.format)
        }
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
