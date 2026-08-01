use serde::{Deserialize, Serialize};

/// Database connection settings, shared by the backend and the worker.
///
/// `dsn` is required and is provided via the `MALPROBE_DATABASE_DSN`
/// environment variable (see `.env`); the other fields have sensible
/// defaults in `malprobe.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub dsn: String,

    pub slow_statements_logging_threshold: Option<u64>,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
}
