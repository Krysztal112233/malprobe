use std::time::Duration;

use malprobe_common::error::Error;
use malprobe_config::DatabaseConfig;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tap::Pipe;
use tracing::error;

/// Create a database connection from the given configuration.
///
/// Applies pool size and slow-query logging settings before connecting.
pub async fn connect(config: &DatabaseConfig) -> Result<DatabaseConnection, Error> {
    let options = ConnectOptions::new(&config.dsn)
        .pipe_borrow_mut(|it| match config.slow_statements_logging_threshold {
            Some(micros) => it.sqlx_slow_statements_logging_settings(
                log::LevelFilter::Warn,
                Duration::from_micros(micros),
            ),
            _ => it,
        })
        .pipe_borrow_mut(|it| match config.max_connections {
            Some(c) => it.max_connections(c),
            _ => it,
        })
        .pipe_borrow_mut(|it| match config.min_connections {
            Some(c) => it.min_connections(c),
            _ => it,
        })
        .pipe_borrow_mut(|it| it.sqlx_logging(false))
        .to_owned();

    Database::connect(options)
        .await
        .inspect_err(|err| {
            error!(
                dsn = %config.dsn,
                error = %err,
                "failed to connect to database"
            )
        })
        .map_err(Error::from)
}
