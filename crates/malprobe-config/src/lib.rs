//! Shared configuration for the malprobe services.
//!
//! All configuration structs are defined here so that the backend and the
//! worker load from the same `malprobe.toml` file with the same
//! `MALPROBE_*` environment overrides.

mod backend;
mod database;
mod worker;

use config::Config;
use serde::de::DeserializeOwned;

pub use backend::BackendConfig;
pub use database::DatabaseConfig;
pub use worker::{WorkerConfig, WorkerSection};

/// Loads a configuration struct from `malprobe.toml` (optional file) with
/// `MALPROBE_*` environment variables taking precedence over the file.
pub(crate) fn load<T>() -> Result<T, config::ConfigError>
where
    T: DeserializeOwned,
{
    Config::builder()
        .add_source(config::File::with_name("malprobe.toml").required(false))
        .add_source(config::Environment::with_prefix("MALPROBE").separator("_"))
        .build()?
        .try_deserialize()
}
