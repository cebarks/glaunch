mod cli;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run(args) => {
            eprintln!("run: not yet implemented");
            std::process::exit(1);
        }
        Command::Profile(args) => {
            eprintln!("profile: not yet implemented");
            std::process::exit(1);
        }
        Command::Tui => {
            eprintln!("tui: not yet implemented");
            std::process::exit(1);
        }
        Command::Info(args) => {
            eprintln!("info: not yet implemented");
            std::process::exit(1);
        }
    }
}
