use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[repr(u8)]
pub enum OrderType {
    Buy,
    Sell,
}

impl From<spark_market_sdk::OrderType> for OrderType {
    fn from(order_type: spark_market_sdk::OrderType) -> Self {
        match order_type {
            spark_market_sdk::OrderType::Buy => OrderType::Buy,
            spark_market_sdk::OrderType::Sell => OrderType::Sell,
        }
    }
}

impl From<OrderType> for spark_market_sdk::OrderType {
    fn from(val: OrderType) -> Self {
        match val {
            OrderType::Buy => spark_market_sdk::OrderType::Buy,
            OrderType::Sell => spark_market_sdk::OrderType::Sell,
        }
    }
}

pub type OrderId = String;

#[derive(Debug, Clone, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Order {
    pub id: OrderId,
    pub user: String,
    pub asset: String,
    pub order_type: OrderType,
    pub amount: u128,
    pub price: u128,
    pub timestamp: u64,
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.price.cmp(&other.price)
    }
}
