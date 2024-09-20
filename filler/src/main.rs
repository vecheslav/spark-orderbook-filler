use dotenv::dotenv;
use price::PriceApi;
use std::{env, sync::Arc, thread};
use tokio::signal::unix::{signal, SignalKind};

use crate::{bot::FillerBot, config::Config, price::CoingeckoApi};

mod bot;
mod config;
mod error;
mod executer;
mod orderbook;
mod price;
mod strategy;
mod trader;
mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    log::info!("Starting instance...");
    let config = Config::load("config.testnet.json")?;
    let price_api = CoingeckoApi::new(
        config.coingecko_host.clone(),
        env::var("COINGECKO_API_KEY").unwrap(),
    );

    let count = thread::available_parallelism()?.get();
    log::info!("THREADS: {}", count);

    let markets = config.markets.clone();

    // ------------------- Start bot -------------------
    let config = Arc::new(config);
    let price_api: Arc<dyn PriceApi> = Arc::new(price_api);

    // Create bots per each market
    let bot = FillerBot::new(markets[0], config.clone(), price_api.clone()).await;

    // Run bot without strategy
    bot.run().await?;

    // TODO: You can run different strategies by api
    bot.start_strategy().await?;

    // ---------------------------------------------------

    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
        _ = sigint.recv() => log::info!("Received signal SIGINT. Shutting down."),
        _ = sigterm.recv() => log::info!("Received signal SIGTERM. Shutting down."),
    }

    Ok(())
}
