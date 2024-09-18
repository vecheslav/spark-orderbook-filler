use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked},
    crypto::SecretKey,
    types::ContractId,
};
use futures::future::join_all;
use spark_market_sdk::SparkMarketContract;
use std::{env, sync::Arc, time::Duration};
use tokio::{sync::RwLock, time};

use crate::{
    config::Config,
    error::Error,
    orderbook::{Orderbook, OrderbookSubscriber},
    price::PriceApi,
    strategy::Strategy,
    trader::Trader,
    types::{Amount, Asset},
};

pub struct FillerBot {
    /// Common configuration
    pub config: Arc<Config>,

    /// Orderbook for the market
    pub orderbook: Arc<RwLock<Orderbook>>,

    /// Last price from external API
    pub last_external_price: Arc<RwLock<Option<u64>>>,

    /// External price API
    pub price_api: Arc<dyn PriceApi>,

    pub base: Asset,
    pub quote: Asset,

    /// Traders
    pub traders: Vec<Trader>,
}

impl FillerBot {
    pub async fn new(
        market_id: ContractId,
        config: Arc<Config>,
        price_api: Arc<dyn PriceApi>,
    ) -> Self {
        log::info!("Initialiaze bot {:?}", market_id);

        // TODO: Remove this hardcoded mnemonic & add multiple wallet support
        let mnemonic = env::var("WALLET_MNEMONIC").unwrap();

        let provider = Provider::connect("testnet.fuel.network").await.unwrap();
        let wallet =
            WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();

        // Secret keys for traders
        let secret_keys = (0..5)
            .map(|i| {
                SecretKey::new_from_mnemonic_phrase_with_path(
                    &mnemonic,
                    &format!("m/44'/60'/0'/{}", i),
                )
                .unwrap()
            })
            .collect::<Vec<_>>();

        // Create traders
        let traders = join_all(
            secret_keys
                .iter()
                .enumerate()
                .map(|(i, &secret_key)| {
                    let wallet =
                        WalletUnlocked::new_from_private_key(secret_key, Some(provider.clone()));
                    Trader::new(i, market_id, wallet)
                })
                .collect::<Vec<_>>(),
        )
        .await;

        let market_contract = SparkMarketContract::new(market_id, wallet.clone()).await;

        // Get market base and quote assets
        let (base, base_decimals, quote, quote_balance, ..) =
            market_contract.config().await.unwrap().value;

        let orderbook = Orderbook::new();

        Self {
            config,
            base: Asset::new(base, base_decimals as u8),
            quote: Asset::new(quote, quote_balance as u8),
            orderbook: Arc::new(RwLock::new(orderbook)),
            last_external_price: Arc::new(RwLock::new(None)),
            price_api,
            traders,
        }
    }

    /// Run the bot with traders
    /// 1. Run orderbook subscriber
    /// 2. Start syncing external price
    /// 3. Run traders
    pub async fn run(&self) -> Result<(), Error> {
        log::info!("Running bot...");

        // Run orderbook subscriber
        let orderbook = self.orderbook.clone();
        let subscriber = OrderbookSubscriber::new(&self.config);
        tokio::spawn(async move {
            if let Err(e) = subscriber.start(orderbook).await {
                log::error!("Error while running orderbook subscriber: {}", e);
            }
        });

        // Start syncing price
        let price_ids = (
            self.config.assets[&self.base.id].price_id.clone(),
            self.config.assets[&self.quote.id].price_id.clone(),
        );
        self.start_sync_external_price(self.price_api.clone(), price_ids)
            .await;

        // Run traders
        for trader in &self.traders {
            trader.run().await;
        }

        Ok(())
    }

    /// Start the strategy separately
    pub async fn start_strategy(&self) -> Result<(), Error> {
        log::info!("Starting strategy...");

        let strategy = Strategy::new(self.base.clone(), self.quote.clone(), self.config.interval);

        // Start strategy per each trader
        for trader in &self.traders {
            strategy
                .start(
                    self.orderbook.clone(),
                    self.last_external_price.clone(),
                    trader.signal_tx.clone(),
                )
                .await;
        }

        Ok(())
    }

    pub async fn stop_strategy(&self) -> Result<(), Error> {
        todo!();
    }

    pub async fn start_sync_external_price(
        &self,
        price_api: Arc<dyn PriceApi>,
        ids: (String, String),
    ) {
        let decimals = self.quote.decimals;
        let last_external_price = self.last_external_price.clone();

        tokio::spawn(async move {
            loop {
                // Get prices for both assets in usd
                let prices = price_api.prices(&[&ids.0, &ids.1]).await.unwrap();

                // Lock and unlock before sleep
                {
                    let mut price = last_external_price.write().await;
                    // Calculate the price of the base asset in terms of the quote asset
                    *price = Some(*Amount::from_readable(prices[0] / prices[1], decimals));
                    // log::info!("EXTERNAL PRICE: {:?}", price);
                }

                // TODO: Sync price every 10 seconds (update when change to pro plan)
                time::sleep(Duration::from_secs(10)).await;
            }
        });
    }
}
