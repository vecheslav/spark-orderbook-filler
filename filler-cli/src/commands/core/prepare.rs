use crate::commands::utils::{setup, validate_contract_id, ETH};
use clap::Args;
use fuels::{
    accounts::{Account, ViewOnlyAccount},
    types::{transaction::TxPolicies, AssetId, Identity},
};
use multiasset_sdk::MultiAssetContract;
use spark_market_sdk::SparkMarketContract;
use std::str::FromStr;
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

#[derive(Args, Clone)]
#[command(about = "Prepare traders")]
pub(crate) struct PrepareCommand {
    /// The contract id of the multiasset
    #[clap(long)]
    pub(crate) multiasset_id: String,

    /// The contract id of the market
    #[clap(long)]
    pub(crate) market_id: String,

    /// The number of traders
    /// Ex. 32
    #[clap(long)]
    pub(crate) traders_num: usize,

    /// Topup only gas
    #[clap(long)]
    pub(crate) only_gas: bool,

    /// The gas amount
    /// Ex. 10000000
    #[clap(long)]
    pub(crate) gas_amount: u64,

    /// The URL to query
    /// Ex. testnet.fuel.network
    #[clap(long)]
    pub(crate) rpc: String,
}

impl PrepareCommand {
    pub(crate) async fn run(&self) -> anyhow::Result<()> {
        let (wallet, traders) = setup(&self.rpc, self.traders_num).await?;
        let market_id = validate_contract_id(&self.market_id)?;
        let multiasset_id = validate_contract_id(&self.multiasset_id)?;

        // Connect to the deployed contracts via the rpc
        let multiasset_contract = MultiAssetContract::new(multiasset_id, wallet.clone()).await;
        let market_contract = SparkMarketContract::new(market_id, wallet.clone()).await;

        let (base, _base_decimals, quote, _quote_balance, ..) =
            market_contract.config().await.unwrap().value;

        let eth = AssetId::from_str(ETH).unwrap();

        let eth_balance = wallet.get_asset_balance(&eth).await?;
        println!("Main wallet balance: {:#?}", eth_balance);

        // Deposit base asset

        // Deposit base asset
        let amount = 100_000_000_000u64;
        // let gas_amount = 100_000_000u64;

        let retry_strategy = ExponentialBackoff::from_millis(10).map(jitter).take(3);

        for (i, trader) in traders.iter().enumerate() {
            let address = trader.address();
            let identity = Identity::from(address);

            // Topup gas for trader
            wallet
                .transfer(address, self.gas_amount, eth, TxPolicies::default())
                .await?;
            println!("{} / Topped up gas for {:?}", i, identity);

            if self.only_gas {
                continue;
            }

            // Mint base and quote assets
            Retry::spawn(retry_strategy.clone(), || async {
                multiasset_contract.mint(identity, &base, amount).await
            })
            .await?;

            Retry::spawn(retry_strategy.clone(), || async {
                multiasset_contract.mint(identity, &quote, amount).await
            })
            .await?;

            println!("{} / Minted {} to {:?}", i, amount, identity);

            // Deposit base and quote assets
            let market_contract = SparkMarketContract::new(market_id, trader.clone()).await;

            Retry::spawn(retry_strategy.clone(), || async {
                market_contract.deposit(amount, base).await
            })
            .await?;
            Retry::spawn(retry_strategy.clone(), || async {
                market_contract.deposit(amount, quote).await
            })
            .await?;
            println!("{} / Deposited {} to {:?}", i, amount, identity);
        }

        Ok(())
    }
}
