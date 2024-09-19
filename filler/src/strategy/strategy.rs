use rand::Rng;
use std::{cmp, sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::JoinHandle, time};

use crate::{
    executer::SignalMessage,
    orderbook::{OrderType, Orderbook},
    types::{Amount, Asset, Sender},
};

/// Simple strategy
pub struct Strategy {
    /// Interval in milliseconds
    pub interval: u64,
    pub base: Asset,
    pub quote: Asset,
}

impl Strategy {
    /// Create a new strategy
    pub fn new(base: Asset, quote: Asset, interval: u64) -> Self {
        Self {
            base,
            quote,
            interval,
        }
    }

    /// Run the strategy
    pub async fn start(
        &self,
        orderbook: Arc<RwLock<Orderbook>>,
        last_external_price: Arc<RwLock<Option<u64>>>,
        signal_tx: Sender<SignalMessage>,
    ) -> JoinHandle<()> {
        let interval = self.interval;
        let base = self.base.clone();
        let quote = self.quote.clone();

        tokio::spawn(async move {
            // ...
            loop {
                time::sleep(Duration::from_millis(interval)).await;

                let last_external_price = last_external_price.read().await;
                if last_external_price.is_none() {
                    log::info!("No external price, skipping...");
                    continue;
                }

                let orderbook = orderbook.read().await;
                let mut rng = rand::thread_rng();
                // Random strategy for now
                let (order_type, price) = if rng.gen_bool(0.5) {
                    let price = orderbook.best_ask().unwrap().price as u64;
                    log::info!("Best ask {}", price);
                    (
                        OrderType::Buy,
                        cmp::min(price, last_external_price.unwrap()),
                    )
                } else {
                    let price = orderbook.best_bid().unwrap().price as u64;
                    log::info!("Best bid {}", price);
                    (
                        OrderType::Sell,
                        cmp::max(price, last_external_price.unwrap()),
                    )
                };
                let amount = Amount::from_readable(rng.gen_range(0.00001..0.0001), base.decimals);

                let signal = SignalMessage {
                    order_type,
                    base: base.clone(),
                    quote: quote.clone(),
                    amount,
                    price,
                };

                if signal_tx.is_closed() {
                    log::info!("Signal channel closed, stopping strategy...");
                    break;
                }

                if let Err(e) = signal_tx.send(signal) {
                    log::error!("Error sending signal: {:?}", e);
                }
            }
        })
    }
}