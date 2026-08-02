use serde::{Deserialize, Serialize};

/// ClamAV / clamd connection settings.
///
/// A standalone, reusable config struct (so any service that talks to clamd
/// can mount it); the worker mounts it under `[worker.clamav]`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClamavConfig {
    /// Address of the clamd INSTREAM endpoint (`host:port`).
    #[serde(default = "default_clamd_addr")]
    pub clamd_addr: String,

    /// Timeout for one clamd INSTREAM scan, in seconds.
    #[serde(default = "default_clamd_timeout_seconds")]
    pub clamd_timeout_seconds: u64,
}

fn default_clamd_addr() -> String {
    "127.0.0.1:3310".to_owned()
}

fn default_clamd_timeout_seconds() -> u64 {
    120
}
