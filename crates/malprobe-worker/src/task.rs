use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Scan task payload enqueued by the backend into the pgmq queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanTask {
    /// Primary key of the `files` table row to scan.
    pub file_id: Uuid,
}
