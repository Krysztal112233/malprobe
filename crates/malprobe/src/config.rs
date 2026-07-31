use config::Config;
use malprobe_common::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub addr: String,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub dsn: String,

    pub slow_statements_logging_threshold: Option<u64>,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,

    pub migrate: bool,
}

impl BackendConfig {
    pub fn new() -> Result<Self, Error> {
        Ok(Config::builder()
            .add_source(config::File::with_name("malprobe.toml").required(false))
            .add_source(config::Environment::with_prefix("MALPROBE").separator("_"))
            .build()?
            .try_deserialize()?)
    }
}
