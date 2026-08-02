//! Shared configuration for the malprobe services.
//!
//! All configuration structs are defined here so that the backend and the
//! worker load from the same `malprobe.toml` file with the same
//! `MALPROBE_*` environment overrides.

mod backend;
mod clamav;
mod database;
mod worker;

use config::Config;
use serde::de::DeserializeOwned;

pub use backend::BackendConfig;
pub use clamav::ClamavConfig;
pub use database::DatabaseConfig;
pub use worker::{WorkerConfig, WorkerSection};

/// Loads a configuration struct from `malprobe.toml` (optional file) with
/// `MALPROBE_*` environment variables taking precedence over the file.
///
/// Environment keys use a double underscore between nesting levels so that
/// single underscores inside field names survive the mapping, e.g.
/// `MALPROBE__DATABASE__DSN` → `database.dsn`,
/// `MALPROBE__WORKER__CLAMAV__CLAMD_ADDR` → `worker.clamav.clamd_addr`,
/// `MALPROBE__WORKER__VT_SECONDS` → `worker.vt_seconds` and
/// `MALPROBE__ADDR` → `addr` (top-level field).
pub(crate) fn load<T>() -> Result<T, config::ConfigError>
where
    T: DeserializeOwned,
{
    Config::builder()
        .add_source(config::File::with_name("malprobe.toml").required(false))
        .add_source(config::Environment::with_prefix("MALPROBE").separator("__"))
        .build()?
        .try_deserialize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Double underscores separate nesting levels; single underscores inside
    /// field names (and top-level fields) must survive the mapping.
    #[test]
    fn environment_maps_nested_fields_and_top_level_fields() {
        let mut env = HashMap::new();
        env.insert("MALPROBE__ADDR".into(), "9.9.9.9:9000".into());
        env.insert("MALPROBE__DATABASE__DSN".into(), "postgresql://x".into());
        env.insert("MALPROBE__WORKER__VT_SECONDS".into(), "5".into());
        env.insert(
            "MALPROBE__WORKER__CLAMAV__CLAMD_ADDR".into(),
            "1.2.3.4:99".into(),
        );

        let config: WorkerConfig = Config::builder()
            .add_source(
                config::Environment::with_prefix("MALPROBE")
                    .separator("__")
                    .source(Some(env)),
            )
            .add_source(config::File::with_name("malprobe.toml").required(false))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();

        assert_eq!(config.worker.vt_seconds, 5);
        assert_eq!(config.worker.clamav.clamd_addr, "1.2.3.4:99");
        assert_eq!(config.database.dsn, "postgresql://x");
    }

    /// `MALPROBE__ADDR` maps to the top-level `addr` field of `BackendConfig`.
    #[test]
    fn environment_maps_top_level_fields() {
        let mut env = HashMap::new();
        env.insert("MALPROBE__ADDR".into(), "9.9.9.9:9000".into());
        env.insert("MALPROBE__DATABASE__DSN".into(), "postgresql://x".into());

        let config: BackendConfig = Config::builder()
            .add_source(
                config::Environment::with_prefix("MALPROBE")
                    .separator("__")
                    .source(Some(env)),
            )
            .add_source(config::File::with_name("malprobe.toml").required(false))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();

        assert_eq!(config.addr, "9.9.9.9:9000");
    }
}
