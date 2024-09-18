use serde::Deserialize;

use super::{Order, OrderType};
use crate::error::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct OrderEnvio {
    pub id: String,
    pub user: String,
    pub asset: String,
    pub amount: String,
    pub price: String,
    pub timestamp: String,
    pub order_type: OrderType,
    pub status: Option<String>,
    pub asset_type: Option<String>,
    pub db_write_timestamp: Option<String>,
    pub initial_amount: Option<String>,
}

impl OrderEnvio {
    pub fn parse(self) -> Result<Order, Error> {
        let amount = self.amount.parse::<u128>()?;
        let price = self.price.parse::<u128>()?;
        let timestamp = chrono::DateTime::parse_from_rfc3339(&self.timestamp)?.timestamp() as u64;

        Ok(Order {
            id: self.id,
            user: self.user,
            asset: self.asset,
            amount,
            price,
            timestamp,
            order_type: self.order_type,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderDataEnvio {
    #[serde(rename = "ActiveBuyOrder")]
    pub buy: Option<Vec<OrderEnvio>>,
    #[serde(rename = "ActiveSellOrder")]
    pub sell: Option<Vec<OrderEnvio>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderPayloadEnvio {
    pub data: OrderDataEnvio,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderResponseEnvio {
    pub r#type: String,
    pub id: Option<String>,
    pub payload: Option<OrderPayloadEnvio>,
}
