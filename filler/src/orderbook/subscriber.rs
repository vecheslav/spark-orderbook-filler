use futures::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;
use tokio::{net::TcpStream, sync::RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use super::{OrderType, Orderbook};
use crate::{config::Config, error::Error, orderbook::OrderResponseEnvio};

pub type Sink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

pub struct OrderbookSubscriber {
    ws_host: String,
}

impl OrderbookSubscriber {
    pub fn new(config: &Config) -> Self {
        let ws_host = config.indexer_ws_host.clone();

        OrderbookSubscriber { ws_host }
    }

    pub async fn start(&self, orderbook: Arc<RwLock<Orderbook>>) -> Result<(), Error> {
        loop {
            log::info!("Connecting to indexer...");
            let (ws_stream, _) = match connect_async(self.ws_host.to_string()).await {
                Ok(res) => res,
                Err(e) => {
                    log::error!("Error while connecting to indexer: {}", e);

                    // Reconnecting delay
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                    continue;
                }
            };
            let (mut sink, mut ws_stream) = ws_stream.split();

            let mut is_subscribed = false;

            // Init connection
            sink.send(Message::Text(r#"{"type": "connection_init"}"#.into()))
                .await?;

            while let Some(message) = ws_stream.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        if let Ok(res) = serde_json::from_str::<OrderResponseEnvio>(&text) {
                            match res.r#type.as_str() {
                                "ka" => {
                                    log::debug!("KA");
                                    continue;
                                }
                                "connection_ack" => {
                                    if !is_subscribed {
                                        self.subscribe(&mut sink, OrderType::Buy).await?;
                                        self.subscribe(&mut sink, OrderType::Sell).await?;
                                        is_subscribed = true;
                                    }
                                }
                                "data" => {
                                    if let Some(payload) = res.payload {
                                        let mut orderbook = orderbook.write().await;
                                        log::info!("NEW ORDERS");

                                        if let Some(orders) = payload.data.buy {
                                            orderbook.buy.clear();
                                            for order in orders {
                                                orderbook.insert(order.parse()?);
                                            }
                                        }
                                        if let Some(orders) = payload.data.sell {
                                            orderbook.sell.clear();
                                            for order in orders {
                                                orderbook.insert(order.parse()?);
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error while reading message: {}", e);
                    }
                    _ => {}
                }
            }

            // TODO: Rewrite to shutdown gracefully
            self.unsubscribe(&mut sink, OrderType::Buy).await?;
            self.unsubscribe(&mut sink, OrderType::Sell).await?;

            log::info!("Closing connection with indexer...");
            sink.close().await.unwrap();
        }
    }

    pub async fn subscribe(&self, sink: &mut Sink, order_type: OrderType) -> Result<(), Error> {
        let (table_name, order) = match order_type {
            OrderType::Sell => ("ActiveSellOrder", "asc"),
            OrderType::Buy => ("ActiveBuyOrder", "desc"),
        };

        let query = format!(
            r#"subscription {{
            {}(limit: {}, order_by: {{ price: {} }}) {{
                id
                user
                timestamp
                order_type
                amount
                asset
                price
                status
            }}
        }}"#,
            table_name, 25, order
        );

        let message = json!({
            "id": format!("{}", order_type as u8),
            "type": "start",
            "payload": {
                "query": query,
            },
        });

        log::info!("SUBSCRIBE: {:?}", order_type);
        sink.send(Message::Text(message.to_string())).await?;

        Ok(())
    }

    pub async fn unsubscribe(&self, sink: &mut Sink, order_type: OrderType) -> Result<(), Error> {
        let message = json!({
            "id": format!("{}", order_type as u8),
            "type": "stop"
        });

        log::info!("UNSUBSCRIBE: {:?}", order_type);
        sink.send(Message::Text(message.to_string())).await?;

        Ok(())
    }
}
