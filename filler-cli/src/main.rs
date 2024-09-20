mod commands;

use clap::Parser;
use commands::{
    cli::{Cli, Command},
    core::cli::CoreCommands,
    info::cli::InfoCommands,
};

use dotenv::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Command::Core(args) => match args.commands {
            CoreCommands::Prepare(args) => args.run().await,
            CoreCommands::Mint(args) => args.run().await,
        },
        Command::Info(args) => match args.commands {
            InfoCommands::Balances(args) => args.run().await,
        },
    }
}
