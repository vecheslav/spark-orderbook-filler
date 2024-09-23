use rand::Rng;
use std::{cmp, sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::JoinHandle, time};

use crate::{
    operation::{OpenOrderOperation, Operation, OperationMessage},
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
        operation_tx: Sender<OperationMessage>,
        max_amount: f64,
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
                // if orderbook.is
                let mut rng = rand::thread_rng();

                // Random strategy for now
                let price = last_external_price.unwrap();
                let (order_type, price) = if rng.gen_bool(0.5) {
                    (
                        OrderType::Buy,
                        match orderbook.best_ask() {
                            Some(order) => cmp::min(order.price as u64, price),
                            None => price,
                        },
                    )
                } else {
                    (
                        OrderType::Sell,
                        match orderbook.best_ask() {
                            Some(order) => cmp::max(order.price as u64, price),
                            None => price,
                        },
                    )
                };
                let amount_range = (max_amount / 10.0)..max_amount;
                let amount = Amount::from_readable(rng.gen_range(amount_range), base.decimals);

                let message = OperationMessage {
                    operation: Operation::OpenOrder(OpenOrderOperation {
                        order_type,
                        base: base.clone(),
                        quote: quote.clone(),
                        amount,
                        price,
                    }),
                };

                if operation_tx.is_closed() {
                    log::info!("Operation channel closed, stopping strategy...");
                    break;
                }

                if let Err(e) = operation_tx.send(message) {
                    log::error!("Error sending operation: {:?}", e);
                }
            }
        })
    }
}
