use std::collections::HashMap;

use common::{ArbitrageError, Context, MpSc, Worker};
use models::{ExchangeProduct, InternalMessage, OrderBook};

#[derive(Clone)]
pub struct OrderBookManager {
    context: Context,
    order_books: HashMap<ExchangeProduct, OrderBook>,
    producer: MpSc<InternalMessage>,
}


impl OrderBookManager {
    pub fn new(context: Context, producer: MpSc<InternalMessage>) -> Self {
        Self { context, order_books: HashMap::new(), producer }
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
                                log::info!("received order book update: {:?}", order_book_update);
                                let order_book = match order_book_manager.order_books.get_mut(&order_book_update.exchange_product) {
                                    Some(order_book) => order_book,
                                    None => {
                                        order_book_manager.order_books.insert(order_book_update.exchange_product.clone(), OrderBook::new(&order_book_update.exchange_product));
                                        order_book_manager.order_books.get_mut(&order_book_update.exchange_product).unwrap()
                                    }
                                };
                                order_book.update(order_book_update);

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
