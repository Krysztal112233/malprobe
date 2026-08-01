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

    #[serde(default = "default_download_queue_name")]
    pub download_queue_name: String,

    #[serde(default = "default_download_concurrency")]
    pub download_concurrency: u32,

    #[serde(default = "default_enrich_queue_name")]
    pub enrich_queue_name: String,

    #[serde(default = "default_enrich_concurrency")]
    pub enrich_concurrency: u32,

    /// Directory where downloaded file bytes are stored, keyed by file id.
    #[serde(default = "default_storage_root")]
    pub storage_root: String,

    /// Address of the clamd INSTREAM endpoint (`host:port`).
    #[serde(default = "default_clamd_addr")]
    pub clamd_addr: String,

    /// Timeout for one clamd INSTREAM scan, in seconds.
    #[serde(default = "default_clamd_timeout_seconds")]
    pub clamd_timeout_seconds: u64,

    /// Maximum number of queue reads before a failing scan is marked as
    /// permanently failed instead of being retried again.
    #[serde(default = "default_max_read_ct")]
    pub max_read_ct: u32,

    /// Upper bound for downloaded file bytes; larger responses are rejected
    /// (matches clamd's INSTREAM size limit, keeping memory bounded).
    #[serde(default = "default_max_download_bytes")]
    pub max_download_bytes: u64,

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

fn default_download_queue_name() -> String {
    "download".to_owned()
}

fn default_download_concurrency() -> u32 {
    2
}

fn default_enrich_queue_name() -> String {
    "enrich".to_owned()
}

fn default_enrich_concurrency() -> u32 {
    2
}

fn default_storage_root() -> String {
    "./storage".to_owned()
}

fn default_clamd_addr() -> String {
    "127.0.0.1:3310".to_owned()
}

fn default_clamd_timeout_seconds() -> u64 {
    120
}

fn default_max_read_ct() -> u32 {
    5
}

fn default_max_download_bytes() -> u64 {
    25 * 1024 * 1024
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
