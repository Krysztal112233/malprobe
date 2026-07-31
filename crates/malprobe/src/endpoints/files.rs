use axum::http::StatusCode;

/// Upload a file for scanning.
#[utoipa::path(
    post,
    path = "/files",
    tag = "files",
    responses((status = NOT_IMPLEMENTED, description = "Not implemented yet"))
)]
pub async fn upload() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

/// Get a scan report by file ID.
#[utoipa::path(
    get,
    path = "/files/{id}",
    tag = "files",
    params(("id" = String, Path, description = "File ID")),
    responses((status = NOT_IMPLEMENTED, description = "Not implemented yet"))
)]
pub async fn get_by_id() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
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
