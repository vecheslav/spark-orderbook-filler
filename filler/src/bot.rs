use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked},
    crypto::SecretKey,
    types::{ContractId, DryRunner},
};
use spark_market_sdk::SparkMarketContract;
use std::{env, sync::Arc, time::Duration};
use tokio::{
    sync::{mpsc::unbounded_channel, Mutex, RwLock},
    time,
};

use crate::{
    config::Config,
    error::Error,
    operation::{OperationManager, OperationMessage},
    orderbook::{Orderbook, OrderbookSubscriber},
    price::PriceApi,
    strategy::Strategy,
    types::{Amount, Asset, Receiver, Sender},
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
    pub traders: Vec<WalletUnlocked>,
    pub next_trader: Arc<Mutex<usize>>,

    pub market_contract: Arc<RwLock<SparkMarketContract>>,
    pub operation_manager: Arc<OperationManager>,
    pub operation_tx: Sender<OperationMessage>,
    pub operation_rx: Receiver<OperationMessage>,

    pub submit_tx: Sender<bool>,
    pub submit_rx: Receiver<bool>,
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
        // Trader set from 0
        let trader_set = env::var("TRADER_SET").unwrap().parse::<usize>().unwrap() - 1;
        log::info!("TRADER_SET: {}", trader_set);

        let provider = Provider::connect("testnet.fuel.network").await.unwrap();
        // let consensus_parameters = provider.consensus_parameters();
        // log::info!("Consensus parameters: {:?}", consensus_parameters);

        let wallet =
            WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();

        // Generate trader wallets
        let trader_offset = trader_set * config.traders_num;
        let traders = (trader_offset..trader_offset + config.traders_num)
            .map(|i| {
                let secret_key = SecretKey::new_from_mnemonic_phrase_with_path(
                    &mnemonic,
                    &format!("m/44'/60'/0'/{}", i),
                )
                .unwrap();
                WalletUnlocked::new_from_private_key(secret_key, Some(provider.clone()))
            })
            .collect::<Vec<_>>();

        let market_contract = SparkMarketContract::new(market_id, wallet.clone()).await;

        // Get market base and quote assets
        let (base, base_decimals, quote, quote_balance, ..) =
            market_contract.config().await.unwrap().value;

        let orderbook = Orderbook::new();

        // Initialize the operation channel & manager
        let (operation_tx, operation_rx) = unbounded_channel::<OperationMessage>();
        let (submit_tx, submit_rx) = unbounded_channel::<bool>();
        let operation_manager = OperationManager::new(config.multicall_size);

        Self {
            config,
            base: Asset::new(base, base_decimals as u8),
            quote: Asset::new(quote, quote_balance as u8),
            orderbook: Arc::new(RwLock::new(orderbook)),
            last_external_price: Arc::new(RwLock::new(None)),
            price_api,
            traders,
            next_trader: Arc::new(Mutex::new(0)),
            market_contract: Arc::new(RwLock::new(market_contract)),
            operation_manager: Arc::new(operation_manager),
            operation_tx: Arc::new(operation_tx),
            operation_rx: Arc::new(Mutex::new(operation_rx)),
            submit_tx: Arc::new(submit_tx),
            submit_rx: Arc::new(Mutex::new(submit_rx)),
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

        self.start_collect_operations().await;
        self.start_process_operations().await;

        Ok(())
    }

    pub async fn start_collect_operations(&self) {
        let operation_manager = self.operation_manager.clone();
        let operation_rx = self.operation_rx.clone();
        let submit_tx = self.submit_tx.clone();
        let multicall_size = self.config.multicall_size;

        // Start handle operations
        tokio::spawn(async move {
            while let Some(message) = operation_rx.lock().await.recv().await {
                let total_operations = operation_manager.add(&message).await;

                if total_operations >= multicall_size {
                    log::debug!("TOTAL: {}", total_operations);

                    if let Err(e) = submit_tx.send(true) {
                        log::error!("{:?}", e);
                    }
                }
            }
        });
    }

    pub async fn start_process_operations(&self) {
        let submit_rx = self.submit_rx.clone();

        let operation_manager = self.operation_manager.clone();
        let market_contract = self.market_contract.clone();
        let traders = self.traders.clone();
        let next_trader = self.next_trader.clone();

        tokio::spawn(async move {
            while submit_rx.lock().await.recv().await.is_some() {
                let mut next_trader = next_trader.lock().await;

                // Select trader
                let trader = traders[*next_trader].clone();

                // Move turn to the next trader
                *next_trader = (*next_trader + 1) % traders.len();

                let operation_manager = operation_manager.clone();
                let market_contract = market_contract.clone();

                tokio::spawn(async move {
                    operation_manager.process(&trader, &market_contract).await;
                });
            }
        });
    }

    /// Start the strategy separately
    pub async fn start_strategy(&self) -> Result<(), Error> {
        log::info!("Starting strategy...");

        let strategy = Strategy::new(self.base.clone(), self.quote.clone(), self.config.interval);

        strategy
            .start(
                self.orderbook.clone(),
                self.last_external_price.clone(),
                self.operation_tx.clone(),
                self.config.max_amount,
            )
            .await;

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
                    log::info!("EXTERNAL PRICE: {:?}", price);
                }

                // TODO: Sync price every 5 seconds (update when change to pro plan)
                time::sleep(Duration::from_secs(5)).await;
            }
        });
    }
}
