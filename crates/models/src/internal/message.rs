use rust_decimal::Decimal;

use super::{Exchange, Product, Side};

pub enum InternalMessage {
    OrderBookUpdate(OrderBookUpdate),
}

pub struct OrderBookUpdate {
    pub product: Product,
    pub exchange: Exchange,
    pub price: Decimal,
    pub amount: Decimal,
    pub side: Side,
}
