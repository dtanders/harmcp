mod cli;
mod error;
mod har;

use clap::Parser;

fn main() {
    let _cli = cli::Cli::parse();
}
