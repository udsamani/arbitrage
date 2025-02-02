use std::collections::HashMap;
use std::env::vars;

use config::{builder::DefaultState, Config, ConfigBuilder, Environment};

pub type CfgBuilder = ConfigBuilder<DefaultState>;

/// Create a new configuration builder
///
/// It updates the environment variables from the provided path if it exists
///
/// Variables are read from the environment variables in any case.
pub fn create_config(env_path: &str) -> CfgBuilder {
    dotenvy::from_path_override(env_path).ok();
    let env_vars = vars().collect::<HashMap<String, String>>();
    let source = Environment::default()
        .source(Some(env_vars));
    Config::builder().add_source(source)
}
