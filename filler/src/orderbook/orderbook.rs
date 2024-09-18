use std::collections::BTreeMap;

use super::{Order, OrderId, OrderType};

#[derive(Debug, Clone)]
pub struct Orderbook {
    pub buy: BTreeMap<OrderId, Order>,
    pub sell: BTreeMap<OrderId, Order>,
}

impl Orderbook {
    pub fn new() -> Self {
        Self {
            buy: BTreeMap::new(),
            sell: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, order: Order) {
        log::info!("INSERT ORDER: {:?}, {}", order.order_type, order.id);

        match order.order_type {
            OrderType::Buy => {
                self.buy.insert(order.id.clone(), order);
            }
            OrderType::Sell => {
                self.sell.insert(order.id.clone(), order);
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
        self.buy.values().next()
    }

    pub fn best_ask(&self) -> Option<&Order> {
        self.sell.values().next_back()
    }

    pub fn remove(&mut self, order_id: &OrderId) {
        self.buy.remove(order_id);
        self.sell.remove(order_id);
    }

    pub fn clear(&mut self) {
        self.buy.clear();
        self.sell.clear();
    }
}
