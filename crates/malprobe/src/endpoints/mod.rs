use axum::Json;
use axum::http::StatusCode;
use malprobe_common::Error;
use malprobe_vo::ApiResponse;

/// Newtype wrapper so `IntoResponse` can be implemented for the shared error
/// type (orphan rule forbids implementing it for `malprobe_common::Error`
/// directly). `From<Error>` makes `?` and `Err(...)` conversions automatic.
#[derive(Debug)]
pub struct ApiError(pub Error);

impl From<Error> for ApiError {
    fn from(value: Error) -> Self {
        Self(value)
    }
}

impl From<sea_orm::error::DbErr> for ApiError {
    fn from(value: sea_orm::error::DbErr) -> Self {
        Self(Error::Db(value))
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (code, msg) = match &self.0 {
            Error::UnknownWithCode(code, msg) => (*code, msg.clone()),
            _ => {
                tracing::error!("{self:?}");
                (500, "internal server error".to_owned())
            }
        };

        let status = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(ApiResponse::<()>::error(code, msg))).into_response()
    }
}

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

mod files;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(files::upload))
        .routes(routes!(files::list))
        .routes(routes!(files::get_by_id))
        .routes(routes!(files::get_by_hash))
        .routes(routes!(files::delete))
}

pub type RestResult<T> = Result<Json<ApiResponse<T>>, ApiError>;
