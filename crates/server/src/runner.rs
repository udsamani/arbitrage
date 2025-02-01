use common::{create_config, ArbitrageResult, Context, Runner};
use config::Config;

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
        Ok(self.context.name.clone())
    }

    fn config(&self) -> &Config {
        &self.context.config
    }
}
