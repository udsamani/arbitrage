use config::Config;

use crate::ArbitrageResult;



#[derive(Debug, Clone)]
pub enum AppMesssage {
    /// Clean Exit
    Exit,
    /// Exit on Failure
    ExitOnFailure,
}

pub type AppBroadcaster = tokio::sync::broadcast::Sender<AppMesssage>;



/// Context for the application
///
/// The context is a container for the service configuration.
#[derive(Clone)]
pub struct Context {
    /// Context Name
    pub name: String,

    /// Configuration
    pub config: Config,

    /// Broadcaster
    pub app: AppBroadcaster,
}


impl Context {

    pub fn from_config(config: Config) -> Self {
        let name = config.get_string("app_name").unwrap_or("default".to_string());
        let broadcaster = tokio::sync::broadcast::Sender::new(10);
        Self { name, config, app: broadcaster }
    }

    pub fn with_name(&self, name: &str) -> Self {
        Self { name: name.to_string(), config: self.config.clone(), app: self.app.clone() }
    }

    pub fn with_config(&self, config: Config) -> Self {
        Self { name: self.name.clone(), config, app: self.app.clone() }
    }

    pub fn log_and_exit(&self, message: &str) -> ArbitrageResult<String> {
        log::warn!("{} - {}", self.name, message);
        Ok(self.name.clone())
    }

    pub fn exit(&self) -> bool {
        self.app.send(AppMesssage::Exit).is_ok()
    }

    pub fn log_and_app_exit(&self) -> ArbitrageResult<String> {
        self.log_and_exit("exit signal received")
    }
}
