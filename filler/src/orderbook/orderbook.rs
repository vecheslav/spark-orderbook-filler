use std::collections::BTreeMap;

use super::{Order, OrderType};

#[derive(Debug, Clone)]
pub struct Orderbook {
    pub buy: BTreeMap<u128, Order>,
    pub sell: BTreeMap<u128, Order>,
}

impl Orderbook {
    pub fn new() -> Self {
        Self {
            buy: BTreeMap::new(),
            sell: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, order: Order) {
        match order.order_type {
            OrderType::Buy => {
                self.buy.insert(order.price, order);
            }
            OrderType::Sell => {
                self.sell.insert(order.price, order);
            }
        }
    }

    pub fn get_orders(&self, order_type: OrderType) -> Vec<&Order> {
        match order_type {
            OrderType::Buy => self.buy.values().collect(),
            OrderType::Sell => self.sell.values().collect(),
        }
    }

    pub fn best_bid(&self) -> Option<&Order> {
        self.buy.values().next_back()
    }

    pub fn best_ask(&self) -> Option<&Order> {
        self.sell.values().next()
    }

    pub fn clear(&mut self) {
        self.buy.clear();
        self.sell.clear();
    }
}
