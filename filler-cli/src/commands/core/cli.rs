use clap::Subcommand;

use super::{mint::MintCommand, topup::TopupCommand};

#[derive(Clone, Subcommand)]
pub(crate) enum CoreCommands {
    #[clap(short_flag = 'D')]
    Topup(TopupCommand),

    #[clap(short_flag = 'M')]
    Mint(MintCommand),
}
