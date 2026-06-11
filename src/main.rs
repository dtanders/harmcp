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
        Command::Headers { indices } => {
            commands::detail::run(&cli.file, indices, DetailSection::Headers, &cli.format)
        }
        Command::Body { indices, output } => match output {
            Some(path) => {
                if indices.len() != 1 {
                    Err(error::HarError::Usage(
                        "--output requires a single index".to_string(),
                    ))
                } else {
                    commands::extract::run(&cli.file, indices[0], path)
                }
            }
            None => commands::detail::run(&cli.file, indices, DetailSection::Body, &cli.format),
        },
        Command::Timings { indices } => {
            commands::detail::run(&cli.file, indices, DetailSection::Timings, &cli.format)
        }
        Command::Stack { indices } => {
            commands::detail::run(&cli.file, indices, DetailSection::Stack, &cli.format)
        }
        Command::All { indices } => {
            commands::detail::run(&cli.file, indices, DetailSection::All, &cli.format)
        }
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
