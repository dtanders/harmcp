mod cli;
mod commands;
mod error;
mod filter;
mod har;
mod input;
mod output;

use clap::Parser;
use cli::{Cli, Command, SkillAction};
use output::DetailSection;

fn main() {
    let cli = Cli::parse();
    let result = dispatch(cli);
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn dispatch(cli: Cli) -> error::Result<()> {
    if let Command::Skill(ref args) = cli.command {
        return match args.action {
            SkillAction::Install { global } => commands::skill::run(global),
        };
    }
    let file = cli.file.as_deref().ok_or_else(|| {
        error::HarError::Usage("<FILE> is required".to_string())
    })?;
    match &cli.command {
        Command::List(args) => commands::list::run(file, args, &cli.format),
        Command::Headers { indices } => {
            commands::detail::run(file, indices, DetailSection::Headers, &cli.format)
        }
        Command::Body { indices, output } => match output {
            Some(path) => {
                if indices.len() != 1 {
                    Err(error::HarError::Usage(
                        "--output requires a single index".to_string(),
                    ))
                } else {
                    commands::extract::run(file, indices[0], path)
                }
            }
            None => commands::detail::run(file, indices, DetailSection::Body, &cli.format),
        },
        Command::Timings { indices } => {
            commands::detail::run(file, indices, DetailSection::Timings, &cli.format)
        }
        Command::Stack { indices } => {
            commands::detail::run(file, indices, DetailSection::Stack, &cli.format)
        }
        Command::Cookies { indices } => {
            commands::detail::run(file, indices, DetailSection::Cookies, &cli.format)
        }
        Command::Info { indices } => {
            commands::detail::run(file, indices, DetailSection::Info, &cli.format)
        }
        Command::Ws { indices } => {
            commands::detail::run(file, indices, DetailSection::Ws, &cli.format)
        }
        Command::All { indices } => {
            commands::detail::run(file, indices, DetailSection::All, &cli.format)
        }
        Command::Summary(args) => commands::summary::run(file, args, &cli.format),
        Command::Pages => commands::pages::run(file, &cli.format),
        Command::Skill(_) => unreachable!(),
    }
}
