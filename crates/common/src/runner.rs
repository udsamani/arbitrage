use config::Config;

use crate::ArbitrageResult;

/// Trait for running an app
#[async_trait::async_trait]
pub trait Runner {

    async fn run(&mut self) -> ArbitrageResult<String>;

    fn config(&self) -> &Config;
}


pub fn run_app<R: Runner>(mut runner: R) {
    let config = runner.config();
    let worker_threads = config.get_int("tokio.worker_threads").unwrap_or(4);
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads as usize)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            runner.run().await.unwrap();
        });
}
