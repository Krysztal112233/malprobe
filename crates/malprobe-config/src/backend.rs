use serde::{Deserialize, Serialize};

use crate::DatabaseConfig;

/// Backend service configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub addr: String,
    pub database: DatabaseConfig,
}

impl BackendConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        crate::load()
    }
}
