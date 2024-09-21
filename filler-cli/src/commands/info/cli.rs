use crate::commands::info::balances::BalancesCommand;
use clap::Subcommand;

#[derive(Clone, Subcommand)]
pub(crate) enum InfoCommands {
    /// Query asset balances information
    #[clap(short_flag = 'B')]
    Balances(BalancesCommand),
}
