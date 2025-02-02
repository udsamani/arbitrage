use rust_decimal::Decimal;

use crate::{deribit::DeribitChannelData, okex::{OkexAction, OkexMessage}};

use super::{Exchange, ExchangeProduct, Product};

pub enum InternalMessage {
    OrderBookUpdate(OrderBookUpdate),
}

#[derive(Debug, Clone)]
pub struct OrderBookUpdate {
    pub exchange_product: ExchangeProduct,
    pub bids: Vec<(Decimal, Decimal)>,
    pub asks: Vec<(Decimal, Decimal)>,
}


impl From<DeribitChannelData> for InternalMessage {
    fn from(data: DeribitChannelData) -> Self {
        match data {
            DeribitChannelData::OrderBook(order_book) => {
                let bids = order_book.bids.iter().map(|bid| (bid.price, bid.amount)).collect();
                let asks = order_book.asks.iter().map(|ask| (ask.price, ask.amount)).collect();
                // TODO: handle errors better
                let product = ExchangeProduct {
                    exchange: Exchange::Deribit,
                    product: Product::from_deribit_exchange(&order_book.instrument_name).unwrap(),
                };
                InternalMessage::OrderBookUpdate(OrderBookUpdate {
                    exchange_product: product,
                    bids,
                    asks,
                })
            }
        }
    }
}

impl From<OkexMessage> for InternalMessage {
    fn from(message: OkexMessage) -> Self {
        match message.action {
            OkexAction::Snapshot => {
                let mut asks = Vec::new();
                let mut bids = Vec::new();
                for data in message.data {
                    asks.extend(data.asks.iter().map(|ask| (ask.price, ask.amount)));
                    bids.extend(data.bids.iter().map(|bid| (bid.price, bid.amount)));
                }
                let product = ExchangeProduct {
                    exchange: Exchange::Okex,
                    product: Product::from_okex_exhchange(&message.arg.instance_id).unwrap(),
                };
                InternalMessage::OrderBookUpdate(OrderBookUpdate {
                    exchange_product: product,
                    bids,
                    asks,
                })
            }
            OkexAction::Update => {
                //TODO: handle errors better
                let data = message.data.first().unwrap();
                let bids = data.bids.iter().map(|bid| (bid.price, bid.amount)).collect();
                let asks = data.asks.iter().map(|ask| (ask.price, ask.amount)).collect();
                let product = ExchangeProduct {
                    exchange: Exchange::Okex,
                    product: Product::from_okex_exhchange(&message.arg.instance_id).unwrap(),
                };
                InternalMessage::OrderBookUpdate(OrderBookUpdate {
                    exchange_product: product,
                    bids,
                    asks,
                })
            }
        }
    }
}


#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub product: Product,
    pub buy_exchange: Exchange,
    pub sell_exchange: Exchange,
    pub buy_price: Decimal,
    pub sell_price: Decimal,
    pub size: Decimal,
}
