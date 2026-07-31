use axum::Json;
use malprobe_common::Error;
use malprobe_vo::ApiResponse;
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

// Reserved for the upcoming endpoint implementations.
#[allow(dead_code)]
pub type RestResult<T> = Result<Json<ApiResponse<T>>, Error>;
