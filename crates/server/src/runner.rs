use common::{create_config, ArbitrageResult, Context, MpSc, Runner, Workers};
use config::Config;

use crate::{adapters::{DeribitExchangeAdapter, OkexExchangeAdapter}, manager::OrderBookManager};

pub struct ServerRunner {
    context: Context,
}

impl Default for ServerRunner {
    fn default() -> Self {
        let config = create_config(".env/server.env")
            .build()
            .expect("server config should be created");
        let context = Context::from_config(config);
        Self { context }
    }
}



#[async_trait::async_trait]
impl Runner for ServerRunner {
    async fn run(&mut self) -> ArbitrageResult<String> {
        log::info!("starting arbitrage server");


        let mut internal_message_producer = MpSc::new(5000);
        let order_book_manager = OrderBookManager::new(self.context.with_name("order-book-manager"), internal_message_producer.clone_with_receiver());

        let mut workers = Workers::new(self.context.with_name("arbitrage-workers"), 0);

        workers.add_worker(Box::new(order_book_manager));

        let mut okex_adapter = OkexExchangeAdapter::new(self.context.clone())?;
        let okex_callback = okex_adapter.callback(internal_message_producer.sender());

        workers.add_worker(okex_adapter.worker(okex_callback));

        let mut deribit_adapter = DeribitExchangeAdapter::new(self.context.clone())?;
        let deribit_callback = deribit_adapter.callback(internal_message_producer.sender());

        workers.add_worker(deribit_adapter.worker(deribit_callback));

        workers.run().await
    }

    fn config(&self) -> &Config {
        &self.context.config
    }
}
