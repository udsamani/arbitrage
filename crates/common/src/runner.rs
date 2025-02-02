use config::Config;

use crate::ArbitrageResult;
use tracing_subscriber::{prelude::*, EnvFilter};

/// Trait for running an app
#[async_trait::async_trait]
pub trait Runner {

    async fn run(&mut self) -> ArbitrageResult<String>;

    fn config(&self) -> &Config;
}


pub fn run_app<R: Runner>(mut runner: R) {
    let config = runner.config();
    setup_telemetry(config);
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


fn setup_telemetry(cfg: &Config) {
    let log_formatter =  tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_thread_ids(true)
        .boxed();

    let log_level = cfg.get_string("log_level").unwrap_or("info".to_string());
    let log_filter = EnvFilter::builder()
        .with_default_directive(log_level.parse().unwrap())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(log_filter)
        .with(log_formatter)
        .init();
}
