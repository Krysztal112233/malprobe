use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use malprobe_vo::ApiResponse;

pub mod error;

pub use error::Error;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (code, msg) = match self {
            Error::UnknownWithCode(code, msg) => (code, msg),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                StatusCode::INTERNAL_SERVER_ERROR.to_string(),
            ),
        };

        (StatusCode::OK, Json(ApiResponse::<()>::error(code, msg))).into_response()
    }
}
