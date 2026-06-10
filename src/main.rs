mod cli;
mod error;
mod filter;
mod har;
mod output;

use clap::Parser;

fn main() {
    let _cli = cli::Cli::parse();
}
