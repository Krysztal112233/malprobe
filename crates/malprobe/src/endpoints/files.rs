use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use malprobe_common::Error;
use malprobe_database::helper::files::FileHelper;
use malprobe_database::model::prelude::Files;
use malprobe_database::model::{files, sea_orm_active_enums};
use malprobe_vo::{ApiResponse, FileCreateRequest, FileStatus, FileVO, FileVerdict};
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

/// Get a scan report by SHA-256 hash.
#[utoipa::path(
    get,
    path = "/files/hash/{sha256}",
    tag = "files",
    params(("sha256" = String, Path, description = "SHA-256 hash of the file")),
    responses((status = NOT_IMPLEMENTED, description = "Not implemented yet"))
)]
pub async fn get_by_hash() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

/// List scanned files.
#[utoipa::path(
    get,
    path = "/files",
    tag = "files",
    responses((status = NOT_IMPLEMENTED, description = "Not implemented yet"))
)]
pub async fn list() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
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
