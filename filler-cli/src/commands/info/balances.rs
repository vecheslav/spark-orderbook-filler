use crate::commands::utils::{setup, validate_contract_id, ETH};
use clap::Args;
use fuels::{accounts::ViewOnlyAccount, types::AssetId};
use spark_market_sdk::SparkMarketContract;
use std::str::FromStr;

#[derive(Args, Clone)]
#[command(about = "Query an asset balances")]
pub(crate) struct BalancesCommand {
    /// The contract id of the market
    #[clap(long)]
    pub(crate) contract_id: String,

    /// The number of traders
    /// Ex. 32
    #[clap(long)]
    pub(crate) traders_num: usize,

    /// The URL to query
    /// Ex. testnet.fuel.network
    #[clap(long)]
    pub(crate) rpc: String,
}

impl BalancesCommand {
    pub(crate) async fn run(&self) -> anyhow::Result<()> {
        let (wallet, traders) = setup(&self.rpc, self.traders_num).await?;
        let contract_id = validate_contract_id(&self.contract_id)?;
        let market_contract = SparkMarketContract::new(contract_id, wallet.clone()).await;

        let eth = AssetId::from_str(ETH).unwrap();

        for (i, trader) in traders.iter().enumerate() {
            let account = market_contract
                .account(trader.address().into())
                .await
                .unwrap()
                .value;
            println!("{} / ACCOUNT: {:?}", i, account);

            let eth_balance = trader.get_asset_balance(&eth).await?;
            println!("Eth balance: {:?}", eth_balance);
        }

        Ok(())
    }
}
