use std::collections::BTreeMap;
use rust_decimal::Decimal;
use super::{ExchangeProduct, OrderBookUpdate};


#[derive(Debug, Clone)]
pub struct OrderBook {
    pub exchange_product: ExchangeProduct,
    pub bids: BTreeMap<Decimal, Decimal>,
    pub asks: BTreeMap<Decimal, Decimal>,
}


impl OrderBook {
    pub fn new(exchange_product: &ExchangeProduct) -> Self {
        Self { exchange_product: exchange_product.clone(), bids: BTreeMap::new(), asks: BTreeMap::new() }
    }

    pub fn best_bid(&self) -> Option<(Decimal, Decimal)> {
        self.bids.last_key_value().map(|(price, size)| (*price, *size))
    }

    pub fn best_ask(&self) -> Option<(Decimal, Decimal)> {
        self.asks.first_key_value().map(|(price, size)| (*price, *size))
    }

    pub fn add_bid(&mut self, price: Decimal, size: Decimal) {
        if size.is_zero() {
            self.bids.remove(&price);
        } else {
            self.bids.insert(price, size);
        }
    }

    pub fn add_ask(&mut self, price: Decimal, size: Decimal) {
        if size.is_zero() {
            self.asks.remove(&price);
        } else {
            self.asks.insert(price, size);
        }
    }

    pub fn update(&mut self, order_book_update: OrderBookUpdate) {
        for (price, size) in order_book_update.bids {
            self.add_bid(price, size);
        }
        for (price, size) in order_book_update.asks {
            self.add_ask(price, size);
        }
    }
}
