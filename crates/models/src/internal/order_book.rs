use std::collections::BTreeMap;

use rust_decimal::Decimal;

use super::{Exchange, Product};


#[derive(Debug, Clone)]
pub struct OrderBook {
    pub product: Product,
    pub bids: BTreeMap<Decimal, Decimal>,
    pub asks: BTreeMap<Decimal, Decimal>,
    pub exchange: Exchange,
}


impl OrderBook {
    pub fn new(product: Product, exchange: Exchange) -> Self {
        Self { product, bids: BTreeMap::new(), asks: BTreeMap::new(), exchange }
    }

    pub fn add_bid(&mut self, price: Decimal, size: Decimal) {
        self.bids.insert(price, size);
    }

    pub fn add_ask(&mut self, price: Decimal, size: Decimal) {
        self.asks.insert(price, size);
    }
}
