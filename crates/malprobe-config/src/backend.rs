use serde::{Deserialize, Serialize};

use crate::DatabaseConfig;

/// Backend service configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub addr: String,
    pub database: DatabaseConfig,

    /// Browser cross-origin policy for the web UI; absent disables CORS.
    #[serde(default)]
    pub cors: CorsConfig,
}

/// CORS policy.
///
/// `"*"` allows any origin (development), a comma-separated list restricts
/// to those origins (production). Absent (or empty) disables CORS entirely,
/// so browser clients are only served when explicitly configured.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CorsConfig {
    #[serde(default)]
    pub allow_origins: Option<String>,
}

impl BackendConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        crate::load()
    }
}
