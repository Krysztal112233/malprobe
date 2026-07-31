use serde::Deserialize;

use crate::DatabaseConfig;

/// Scan worker configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct WorkerConfig {
    pub database: DatabaseConfig,
    pub worker: WorkerSection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkerSection {
    #[serde(default = "default_queue_name")]
    pub queue_name: String,

    #[serde(default = "default_concurrency")]
    pub concurrency: u32,

    #[serde(default = "default_vt_seconds")]
    pub vt_seconds: u64,

    #[serde(default = "default_poll_timeout_ms")]
    pub poll_timeout_ms: u64,

    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,
}

fn default_queue_name() -> String {
    "scan".to_owned()
}

fn default_concurrency() -> u32 {
    2
}

fn default_vt_seconds() -> u64 {
    300
}

fn default_poll_timeout_ms() -> u64 {
    5000
}

fn default_poll_interval_ms() -> u64 {
    500
}

impl WorkerConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        crate::load()
    }
}
