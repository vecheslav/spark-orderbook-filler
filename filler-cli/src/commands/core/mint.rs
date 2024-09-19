use std::str::FromStr;

use crate::commands::utils::{setup, validate_contract_id};
use clap::Args;
use fuels::types::AssetId;
use multiasset_sdk::MultiAssetContract;

#[derive(Args, Clone)]
#[command(about = "Mint base and quote assets")]
pub(crate) struct MintCommand {
    /// The contract id of the multi-asset
    #[clap(long)]
    pub(crate) contract_id: String,

    /// The base
    #[clap(long)]
    pub(crate) base: String,

    /// The quote
    #[clap(long)]
    pub(crate) quote: String,

    /// The amount to mint
    /// Ex. 10000000
    #[clap(long)]
    pub(crate) amount: u64,

    /// The URL to query
    /// Ex. testnet.fuel.network
    #[clap(long)]
    pub(crate) rpc: String,
}

impl MintCommand {
    pub(crate) async fn run(&self) -> anyhow::Result<()> {
        let (wallet, traders) = setup(&self.rpc).await?;
        let contract_id = validate_contract_id(&self.contract_id)?;

        let base = AssetId::from_str(&self.base).expect("Invalid asset");
        let quote = AssetId::from_str(&self.quote).expect("Invalid asset");

        // Connect to the deployed contract via the rpc
        let contract = MultiAssetContract::new(contract_id, wallet.clone()).await;

        contract
            .mint(wallet.address().into(), &base, self.amount)
            .await?;
        contract
            .mint(wallet.address().into(), &quote, self.amount)
            .await?;

        for trader in traders {
            contract
                .mint(trader.address().into(), &base, self.amount)
                .await?;
            contract
                .mint(trader.address().into(), &quote, self.amount)
                .await?;
        }

        println!("Minted {}", self.amount);

        Ok(())
    }
}
