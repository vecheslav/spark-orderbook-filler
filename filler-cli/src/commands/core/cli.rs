use clap::Subcommand;

use super::{mint::MintCommand, prepare::PrepareCommand};

#[derive(Clone, Subcommand)]
pub(crate) enum CoreCommands {
    #[clap(short_flag = 'P')]
    Prepare(PrepareCommand),

    #[clap(short_flag = 'M')]
    Mint(MintCommand),
}
