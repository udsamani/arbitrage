use std::{cmp::min, collections::HashMap};

use common::{ArbitrageError, Context, MpSc, Worker};
use models::{ArbitrageOpportunity, Exchange, ExchangeProduct, InternalMessage, OrderBook, Product};
use rust_decimal::Decimal;
use tokio::sync::broadcast::Sender;

#[derive(Clone)]
pub struct OrderBookManager {
    context: Context,
    order_books: HashMap<ExchangeProduct, OrderBook>,
    producer: MpSc<InternalMessage>,
    broadcaster: Sender<InternalMessage>,
}


impl OrderBookManager {
    pub fn new(context: Context, producer: MpSc<InternalMessage>, broadcaster: Sender<InternalMessage>) -> Self {
        Self { context, order_books: HashMap::new(), producer, broadcaster }
    }

    pub fn check_arbitrage_opportunities(&self, product: &Product) -> Option<ArbitrageOpportunity> {

        // This can be made more efficient instead of hardcoding for two exchanges.
        // It can be made generic for any number of exchanges. However, it would make the code more complex.
        // For the purpose of this project, it is fine to keep it as is.
        let okex_order_book = self.order_books
            .get(&ExchangeProduct { exchange: Exchange::Okex, product: product.clone() })?;
        let deribit_order_book = self.order_books.get(&ExchangeProduct { exchange: Exchange::Deribit, product: product.clone() })?;

        if let (Some(okex_best_ask), Some(deribit_best_bid)) = (okex_order_book.best_ask(), deribit_order_book.best_bid()) {
            let profit = deribit_best_bid.0 - okex_best_ask.0;
            if profit > Decimal::ZERO {
                return Some(ArbitrageOpportunity {
                    product: product.clone(),
                    buy_exchange: Exchange::Okex,
                    sell_exchange: Exchange::Deribit,
                    buy_price: okex_best_ask.0,
                    sell_price: deribit_best_bid.0,
                    size: min(okex_best_ask.1, deribit_best_bid.1),
                });
            }
        }

        if let (Some(okex_best_bid), Some(deribit_best_ask)) = (okex_order_book.best_bid(), deribit_order_book.best_ask()) {
            let profit = okex_best_bid.0 - deribit_best_ask.0;
            if profit > Decimal::ZERO {
                return Some(ArbitrageOpportunity {
                    product: product.clone(),
                    buy_exchange: Exchange::Deribit,
                    sell_exchange: Exchange::Okex,
                    buy_price: deribit_best_ask.0,
                    sell_price: okex_best_bid.0,
                    size: min(okex_best_bid.1, deribit_best_ask.1),
                });
            }
        }

        None

    }
}


impl Worker for OrderBookManager {
    fn spawn(&mut self) -> common::SpawnResult {
        let mut order_book_manager = self.clone();
        let mut receiver = self.producer.receiver().unwrap();

        tokio::spawn(async move {
            let mut app = order_book_manager.context.app.subscribe();
            loop {
                tokio::select! {
                    _ = app.recv() => {
                        return Err(ArbitrageError::Exit);
                    }

                    result = receiver.recv() => {
                        match result {
                            Some(InternalMessage::OrderBookUpdate(order_book_update)) => {
                                // Entry api for rust hashmap creates a new copy of the key even it already exists
                                // hence we try to avoid it.
                                let order_book = match order_book_manager.order_books.get_mut(&order_book_update.exchange_product) {
                                    Some(order_book) => order_book,
                                    None => {
                                        order_book_manager.order_books.insert(order_book_update.exchange_product.clone(), OrderBook::new(&order_book_update.exchange_product));
                                        order_book_manager.order_books.get_mut(&order_book_update.exchange_product).unwrap()
                                    }
                                };
                                let product = order_book_update.exchange_product.product.clone();
                                order_book.update(order_book_update);
                                let arbitrage_opportunity = order_book_manager.check_arbitrage_opportunities(&product);
                                if let Some(arbitrage_opportunity) = arbitrage_opportunity {
                                    log::info!("arbitrage opportunity: {:?}", arbitrage_opportunity);
                                    match order_book_manager.broadcaster.send(InternalMessage::ArbitrageOpportunity(arbitrage_opportunity)) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            log::error!("error sending arbitrage opportunity to broadcaster: {:?}", e);
                                        }
                                    }
                                }

                            }
                            Some(InternalMessage::ArbitrageOpportunity(_)) => {
                                log::warn!("received arbitrage opportunity from broadcaster, this should not happen");
                            }
                            None => {
                                return Err(ArbitrageError::GenericError("receiver closed".to_string()));
                            }
                        }
                    }

                }
            }
        })
    }
}



#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use chrono::NaiveDate;
    use config::Config;
    use models::{Exchange, ExchangeProduct, OrderBookUpdate, Product};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use tokio::sync::broadcast;
    use super::*;

    fn setup_order_book_manager(product: Product) -> OrderBookManager {
        let context = Context::from_config(Config::default());
        let producer = MpSc::new(100);
        let (broadcaster, _) = broadcast::channel(100);
        let mut order_book_manager = OrderBookManager::new(context, producer, broadcaster);

        // Setup Order Book for Okex
        let okex_order_book = OrderBook::new(&ExchangeProduct { exchange: Exchange::Okex, product: product.clone() });
        order_book_manager.order_books.insert(ExchangeProduct { exchange: Exchange::Okex, product: product.clone() }, okex_order_book);

        // Setup Order Book for Deribit
        let deribit_order_book = OrderBook::new(&ExchangeProduct { exchange: Exchange::Deribit, product: product.clone() });
        order_book_manager.order_books.insert(ExchangeProduct { exchange: Exchange::Deribit, product: product.clone() }, deribit_order_book);

        order_book_manager
    }

    #[test]
    fn test_arbitrage_okex_buy_deribit_sell() {
        let product = Product::Option {
            underlying: models::CryptoAsset::BTC,
            settlement: models::SettlementAsset::USD,
            strike: Decimal::from_str("90000").unwrap(),
            option_type: models::OptionType::Call,
            expiration: NaiveDate::from_ymd_opt(2025, 2, 21).unwrap(),
        };

        let mut order_book_manager = setup_order_book_manager(product.clone());

        // Setup Order Book for Okex
        let okex_order_book = order_book_manager.order_books.get_mut(&ExchangeProduct { exchange: Exchange::Okex, product: product.clone() }).unwrap();
        okex_order_book.update(OrderBookUpdate {
            exchange_product: ExchangeProduct { exchange: Exchange::Okex, product: product.clone() },
            bids: vec![(dec!(0.018), dec!(5400)), (dec!(0.019), dec!(1000))],
            asks: vec![(dec!(0.015), dec!(1000)), (dec!(0.021), dec!(5400))],
        });

        let deribit_order_book = order_book_manager.order_books.get_mut(&ExchangeProduct { exchange: Exchange::Deribit, product: product.clone() }).unwrap();
        deribit_order_book.update(OrderBookUpdate {
            exchange_product: ExchangeProduct { exchange: Exchange::Deribit, product: product.clone() },
            bids: vec![(dec!(0.018), dec!(5400)), (dec!(0.019), dec!(1000))],
            asks: vec![(dec!(0.020), dec!(1000)), (dec!(0.021), dec!(5400))],
        });

        let arbitrage_opportunity = order_book_manager.check_arbitrage_opportunities(&product).unwrap();
        assert_eq!(arbitrage_opportunity.buy_exchange, Exchange::Okex);
        assert_eq!(arbitrage_opportunity.sell_exchange, Exchange::Deribit);
        assert_eq!(arbitrage_opportunity.product, product);
        assert_eq!(arbitrage_opportunity.buy_price, dec!(0.015));
        assert_eq!(arbitrage_opportunity.sell_price, dec!(0.019));
        assert_eq!(arbitrage_opportunity.size, dec!(1000));
    }

    #[test]
    fn test_arbitrage_deribit_buy_okex_sell() {
        let product = Product::Option {
            underlying: models::CryptoAsset::BTC,
            settlement: models::SettlementAsset::USD,
            strike: Decimal::from_str("90000").unwrap(),
            option_type: models::OptionType::Call,
            expiration: NaiveDate::from_ymd_opt(2025, 2, 21).unwrap(),
        };

        let mut order_book_manager = setup_order_book_manager(product.clone());

        let deribit_order_book = order_book_manager.order_books.get_mut(&ExchangeProduct { exchange: Exchange::Deribit, product: product.clone() }).unwrap();
        deribit_order_book.update(OrderBookUpdate {
            exchange_product: ExchangeProduct { exchange: Exchange::Deribit, product: product.clone() },
            bids: vec![(dec!(0.018), dec!(5400)), (dec!(0.019), dec!(1000))],
            asks: vec![(dec!(0.015), dec!(1000)), (dec!(0.021), dec!(5400))],
        });

        let okex_order_book = order_book_manager.order_books.get_mut(&ExchangeProduct { exchange: Exchange::Okex, product: product.clone() }).unwrap();
        okex_order_book.update(OrderBookUpdate {
            exchange_product: ExchangeProduct { exchange: Exchange::Okex, product: product.clone() },
            bids: vec![(dec!(0.018), dec!(5400)), (dec!(0.019), dec!(1000))],
            asks: vec![(dec!(0.020), dec!(1000)), (dec!(0.021), dec!(5400))],
        });

        let arbitrage_opportunity = order_book_manager.check_arbitrage_opportunities(&product).unwrap();
        assert_eq!(arbitrage_opportunity.buy_exchange, Exchange::Deribit);
        assert_eq!(arbitrage_opportunity.sell_exchange, Exchange::Okex);
        assert_eq!(arbitrage_opportunity.product, product);
        assert_eq!(arbitrage_opportunity.buy_price, dec!(0.015));
        assert_eq!(arbitrage_opportunity.sell_price, dec!(0.019));
        assert_eq!(arbitrage_opportunity.size, dec!(1000));
    }

    #[test]
    fn test_arbitrage_none_for_different_products() {
        let product1 = Product::Option {
            underlying: models::CryptoAsset::BTC,
            settlement: models::SettlementAsset::USD,
            strike: Decimal::from_str("90000").unwrap(),
            option_type: models::OptionType::Call,
            expiration: NaiveDate::from_ymd_opt(2025, 2, 21).unwrap(),
        };

        let product2 = Product::Option {
            underlying: models::CryptoAsset::BTC,
            settlement: models::SettlementAsset::USD,
            strike: Decimal::from_str("90000").unwrap(),
            option_type: models::OptionType::Put,
            expiration: NaiveDate::from_ymd_opt(2025, 2, 21).unwrap(),
        };

        let mut order_book_manager = setup_order_book_manager(product1.clone());

        let okex_exchange_product = ExchangeProduct { exchange: Exchange::Okex, product: product2.clone() };
        order_book_manager.order_books.insert(okex_exchange_product.clone(), OrderBook::new(&okex_exchange_product));

        let arbitrage_opportunity = order_book_manager.check_arbitrage_opportunities(&product2);
        assert!(arbitrage_opportunity.is_none());
    }
}
