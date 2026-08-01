use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use malprobe_common::Error;
use malprobe_database::helper::files::FileHelper;
use malprobe_database::model::prelude::Files;
use malprobe_database::model::{files, sea_orm_active_enums};
use malprobe_vo::{
    ApiResponse, FileCreateRequest, FileStatus, FileVO, FileVerdict, PageInfo, PagedResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::endpoints::RestResult;
use crate::state::AppState;

/// Submit a file by its download URL for scanning. The file bytes are not
/// uploaded here; the worker downloads them from the URL.
#[utoipa::path(
    post,
    path = "/files",
    tag = "files",
    request_body = FileCreateRequest,
    responses(
        (status = OK, description = "File accepted for scanning", body = ApiResponse<FileVO>),
        (status = INTERNAL_SERVER_ERROR, description = "Server error")
    )
)]
pub async fn upload(
    State(state): State<AppState>,
    Json(request): Json<FileCreateRequest>,
) -> RestResult<FileVO> {
    let id = Uuid::now_v7();

    let inserted = Files::create_pending_file(
        id,
        sea_orm_active_enums::FileSourceType::Url,
        request.url,
        &state.database,
    )
    .await?;

    Ok(Json(ApiResponse::new(to_vo(inserted))))
}

/// Get a scan report by file ID.
#[utoipa::path(
    get,
    path = "/files/{id}",
    tag = "files",
    params(("id" = Uuid, Path, description = "File ID")),
    responses(
        (status = OK, description = "Scan report", body = ApiResponse<FileVO>),
        (status = NOT_FOUND, description = "File not found")
    )
)]
pub async fn get_by_id(State(state): State<AppState>, Path(id): Path<Uuid>) -> RestResult<FileVO> {
    let Some(model) = Files::find_by_id(id, &state.database).await? else {
        return Err(Error::UnknownWithCode(404, format!("file {id} not found")).into());
    };

    Ok(Json(ApiResponse::new(to_vo(model))))
}

/// Get all scan reports by SHA-256 hash, newest first.
///
/// The same bytes can be submitted from different sources; every submission
/// keeps its own scan record, so the response is a list and deduplication is
/// left to the caller. (`ApiResponse` flattens struct payloads, so the list
/// rides in the same `PagedResponse` shape the list endpoint uses.)
#[utoipa::path(
    get,
    path = "/files/hash/{sha256}",
    tag = "files",
    params(("sha256" = String, Path, description = "SHA-256 hash of the file")),
    responses(
        (status = OK, description = "All scan reports for this hash, newest first", body = ApiResponse<PagedResponse<FileVO>>),
        (status = INTERNAL_SERVER_ERROR, description = "Server error")
    )
)]
pub async fn get_by_hash(
    State(state): State<AppState>,
    Path(sha256): Path<String>,
) -> RestResult<PagedResponse<FileVO>> {
    let models = Files::find_by_hash(sha256, &state.database).await?;
    Ok(Json(ApiResponse::new(PagedResponse::with_entire(
        models.into_iter().map(to_vo),
    ))))
}

/// Query parameters for the file list endpoint.
#[derive(Debug, Deserialize)]
pub struct ListParams {
    /// 1-based page number.
    #[serde(default = "default_page")]
    pub page: u32,
    /// Page size, capped at 100.
    #[serde(default = "default_size")]
    pub size: u32,
}

fn default_page() -> u32 {
    1
}

fn default_size() -> u32 {
    20
}

/// List scanned files, newest first.
#[utoipa::path(
    get,
    path = "/files",
    tag = "files",
    params(
        ("page" = Option<u32>, Query, description = "1-based page number (default 1)"),
        ("size" = Option<u32>, Query, description = "Page size, max 100 (default 20)"),
    ),
    responses(
        (status = OK, description = "Paged scan reports, newest first", body = ApiResponse<PagedResponse<FileVO>>),
        (status = BAD_REQUEST, description = "Invalid page parameters"),
        (status = INTERNAL_SERVER_ERROR, description = "Server error")
    )
)]
pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> RestResult<PagedResponse<FileVO>> {
    if params.page == 0 {
        return Err(Error::UnknownWithCode(400, "page must be >= 1".to_owned()).into());
    }
    if params.size == 0 || params.size > 100 {
        return Err(
            Error::UnknownWithCode(400, "size must be between 1 and 100".to_owned()).into(),
        );
    }

    let (models, total) =
        Files::find_page(params.page as u64, params.size as u64, &state.database).await?;
    let has_next = (params.page as u64) * (params.size as u64) < total;
    Ok(Json(ApiResponse::new(PagedResponse {
        items: models.into_iter().map(to_vo).collect(),
        page_info: PageInfo {
            has_next,
            total: total as usize,
        },
    })))
}

/// Delete a scanned file.
#[utoipa::path(
    delete,
    path = "/files/{id}",
    tag = "files",
    params(("id" = String, Path, description = "File ID")),
    responses((status = NOT_IMPLEMENTED, description = "Not implemented yet"))
)]
pub async fn delete() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

fn to_vo(model: files::Model) -> FileVO {
    FileVO {
        id: model.id,
        sha256: model.sha256,
        size: model.size,
        mime_type: model.mime_type,
        status: to_status(model.status),
        verdict: model.verdict.map(to_verdict),
        malware_name: model.malware_name,
        details: model.details,
        error: model.error,
        // The entity uses `DateTime<FixedOffset>` (sqlx-postgres); the VO
        // contract exposes UTC.
        created_at: model.created_at.with_timezone(&chrono::Utc),
        updated_at: model.updated_at.with_timezone(&chrono::Utc),
        scanned_at: model.scanned_at.map(|t| t.with_timezone(&chrono::Utc)),
    }
}

fn to_status(status: sea_orm_active_enums::FileStatus) -> FileStatus {
    match status {
        sea_orm_active_enums::FileStatus::Pending => FileStatus::Pending,
        sea_orm_active_enums::FileStatus::Scanning => FileStatus::Scanning,
        sea_orm_active_enums::FileStatus::Completed => FileStatus::Completed,
        sea_orm_active_enums::FileStatus::Failed => FileStatus::Failed,
    }
}

fn to_verdict(verdict: sea_orm_active_enums::FileVerdict) -> FileVerdict {
    match verdict {
        sea_orm_active_enums::FileVerdict::Clean => FileVerdict::Clean,
        sea_orm_active_enums::FileVerdict::Suspicious => FileVerdict::Suspicious,
        sea_orm_active_enums::FileVerdict::Malicious => FileVerdict::Malicious,
        sea_orm_active_enums::FileVerdict::Error => FileVerdict::Error,
    }
}
