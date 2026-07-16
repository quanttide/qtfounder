mod git;
mod health;
mod util;

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    #[command(subcommand)]
    Health(health::Commands),
}

fn main() {
    match Cli::parse().command {
        CliCommand::Health(cmd) => health::dispatch(cmd),
    }
}
