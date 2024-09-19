use crate::commands::utils::{setup, validate_contract_id};
use clap::Args;
use fuels::accounts::ViewOnlyAccount;
use spark_market_sdk::SparkMarketContract;

#[derive(Args, Clone)]
#[command(about = "Query an asset balances")]
pub(crate) struct BalancesCommand {
    /// The contract id of the market
    #[clap(long)]
    pub(crate) contract_id: String,

    /// The URL to query
    /// Ex. testnet.fuel.network
    #[clap(long)]
    pub(crate) rpc: String,
}

impl BalancesCommand {
    pub(crate) async fn run(&self) -> anyhow::Result<()> {
        let (wallet, traders) = setup(&self.rpc).await?;
        let contract_id = validate_contract_id(&self.contract_id)?;

        let market_contract = SparkMarketContract::new(contract_id, wallet.clone()).await;
        let (base, _base_decimals, quote, _quote_balance, ..) =
            market_contract.config().await.unwrap().value;

        let account = market_contract
            .account(wallet.address().into())
            .await
            .unwrap()
            .value;
        let base_balance = wallet.get_asset_balance(&base).await?;
        let quote_balance = wallet.get_asset_balance(&quote).await?;
        println!("BALANCES: {:?}", (base_balance, quote_balance));
        println!("ACCOUNT: {:?}", account);

        for (i, trader) in traders.iter().enumerate() {
            let account = market_contract
                .account(trader.address().into())
                .await
                .unwrap()
                .value;

            let base_balance = trader.get_asset_balance(&base).await?;
            let quote_balance = trader.get_asset_balance(&quote).await?;
            println!("BALANCES: {:?}", (base_balance, quote_balance));
            println!("{} / ACCOUNT: {:?}", i, account);
        }

        Ok(())
    }
}
