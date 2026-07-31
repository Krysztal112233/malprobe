pub mod file;

pub use file::{FileStatus, FileVO, FileVerdict};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, Default, ToSchema)]
pub struct Empty {}

#[derive(Debug, Deserialize, Serialize, Default, ToSchema)]
pub struct ApiResponse<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    payload: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn new(payload: T) -> Self {
        Self {
            payload: Some(payload),
            code: None,
            msg: None,
        }
    }

    pub fn error(code: impl Into<u16>, msg: impl Into<String>) -> Self {
        Self {
            code: Some(code.into()),
            msg: Some(msg.into()),
            payload: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PageInfo {
    pub has_next: bool,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PagedResponse<T> {
    pub items: Vec<T>,
    pub page_info: PageInfo,
}

impl<T> PagedResponse<T> {
    pub fn with_entire<I>(data: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let items = data.into_iter().collect::<Vec<_>>();

        let page_info = PageInfo {
            has_next: false,
            total: items.len(),
        };

        Self { items, page_info }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn api_response_skips_missing_fields() {
        let value = serde_json::to_value(ApiResponse::new(json!({ "a": 1 }))).unwrap();

        assert!(value.get("code").is_none());
        assert!(value.get("msg").is_none());
        assert_eq!(value["a"], 1);
    }

    #[test]
    fn api_response_error_fields_are_serialized() {
        let value = serde_json::to_value(ApiResponse::<()>::error(400u16, "bad request")).unwrap();

        assert_eq!(value["code"], 400);
        assert_eq!(value["msg"], "bad request");
    }

    #[test]
    fn paged_response_serializes_page_info() {
        let value = serde_json::to_value(PagedResponse::with_entire(vec![1, 2])).unwrap();

        assert_eq!(value["items"], json!([1, 2]));
        assert_eq!(value["page_info"]["total"], 2);
        assert_eq!(value["page_info"]["has_next"], false);
    }
}
