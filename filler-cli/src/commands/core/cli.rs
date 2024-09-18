use clap::Subcommand;

use super::topup::TopupCommand;

#[derive(Clone, Subcommand)]
pub(crate) enum CoreCommands {
    #[clap(short_flag = 'D')]
    Topup(TopupCommand),
}
