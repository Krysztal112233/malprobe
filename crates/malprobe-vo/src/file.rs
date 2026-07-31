use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FileVO {
    pub id: Uuid,
    pub sha256: String,
    pub size: i64,
    pub mime_type: Option<String>,
    pub status: FileStatus,
    pub verdict: Option<FileVerdict>,
    pub malware_name: Option<String>,
    pub details: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub scanned_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Pending,
    Scanning,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FileVerdict {
    Clean,
    Suspicious,
    Malicious,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[test]
    fn file_vo_serializes_with_api_visible_fields() {
        let vo = FileVO {
            id: Uuid::now_v7(),
            sha256: "abc".to_owned(),
            size: 42,
            mime_type: Some("text/plain".to_owned()),
            status: FileStatus::Pending,
            verdict: None,
            malware_name: None,
            details: None,
            error: None,
            created_at: DateTime::from_timestamp(0, 0).unwrap(),
            updated_at: DateTime::from_timestamp(0, 0).unwrap(),
            scanned_at: None,
        };

        let value = serde_json::to_value(&vo).unwrap();

        assert_eq!(value["mime_type"], "text/plain");
        assert_eq!(value["status"], "pending");
        assert_eq!(value["created_at"], "1970-01-01T00:00:00Z");
        assert!(value.get("storage_path").is_none());
    }

    #[test]
    fn status_and_verdict_use_snake_case() {
        assert_eq!(
            serde_json::to_string(&FileStatus::Scanning).unwrap(),
            "\"scanning\""
        );
        assert_eq!(
            serde_json::to_string(&FileVerdict::Malicious).unwrap(),
            "\"malicious\""
        );
    }
}
