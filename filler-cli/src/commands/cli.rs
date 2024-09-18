use clap::{Args, Parser, Subcommand};

use super::{core::cli::CoreCommands, info::cli::InfoCommands};

#[derive(Parser)]
#[command(about = "Command line parser")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Clone, Subcommand)]
pub(crate) enum Command {
    #[clap(short_flag = 'C')]
    Core(Core),

    #[clap(short_flag = 'I')]
    Info(Info),
}

#[derive(Args, Clone)]
pub(crate) struct Core {
    #[clap(subcommand)]
    pub(crate) commands: CoreCommands,
}

#[derive(Args, Clone)]
pub(crate) struct Info {
    #[clap(subcommand)]
    pub(crate) commands: InfoCommands,
}
