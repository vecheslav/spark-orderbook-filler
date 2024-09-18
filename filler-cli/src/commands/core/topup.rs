use crate::commands::utils::{setup, validate_contract_id};
use clap::Args;
use fuels::accounts::ViewOnlyAccount;
use spark_market_sdk::SparkMarketContract;

#[derive(Args, Clone)]
#[command(about = "Topup base and quote assets on the market")]
pub(crate) struct TopupCommand {
    /// The contract id of the market
    #[clap(long)]
    pub(crate) contract_id: String,

    /// The URL to query
    /// Ex. testnet.fuel.network
    #[clap(long)]
    pub(crate) rpc: String,
}

impl TopupCommand {
    pub(crate) async fn run(&self) -> anyhow::Result<()> {
        let (wallet, traders) = setup(&self.rpc).await?;
        let contract_id = validate_contract_id(&self.contract_id)?;

        let market_contract = SparkMarketContract::new(contract_id, wallet.clone()).await;

        let (base, _base_decimals, quote, _quote_balance, ..) =
            market_contract.config().await.unwrap().value;

        let base_balance = wallet.get_asset_balance(&base).await?;
        let quote_balance = wallet.get_asset_balance(&quote).await?;

        println!(
            "Base balance: {:#?}\nQuote balance: {:#?}",
            base_balance, quote_balance
        );

        // Deposit base asset
        let amount = 100_000_000_000u64;
        market_contract.deposit(amount, base).await?;
        market_contract.deposit(amount, quote).await?;
        println!("Deposited {}", amount);

        for trader in traders {
            let market_contract = SparkMarketContract::new(contract_id, trader.clone()).await;
            market_contract.deposit(amount, base).await?;
            market_contract.deposit(amount, quote).await?;
            println!("Deposited {}", amount);
        }

        Ok(())
    }
}
