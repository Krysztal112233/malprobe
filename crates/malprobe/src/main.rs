use axum::Router;
use axum::http::Method;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE, HeaderValue};
use log::error;
use malprobe_common::error::Error;
use malprobe_config::BackendConfig;
use malprobe_database::helper::files::FileHelper;
use malprobe_database::model::prelude::Files;
use mimalloc::MiMalloc;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::state::AppState;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod endpoints;
mod state;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let config = dbg!(BackendConfig::load().inspect_err(|e| error!("{e}"))?);

    let states = {
        // Migrations are run by the dedicated migration container (see compose.yml
        // `backend-migration`), never at backend startup.
        let database = malprobe_database::setup::connect(&config.database)
            .await
            .inspect_err(|e| error!("{e}"))?;

        // `pgmq.create` is idempotent; the backend must ensure the queues exist
        // because it can run before any worker started.
        Files::ensure_scan_queue(&database)
            .await
            .inspect_err(|e| error!("{e}"))?;
        Files::ensure_download_queue(&database)
            .await
            .inspect_err(|e| error!("{e}"))?;
        Files::ensure_enrich_queue(&database)
            .await
            .inspect_err(|e| error!("{e}"))?;

        AppState { database }
    };

    let (router, openapi) = OpenApiRouter::new()
        .merge(endpoints::router())
        .split_for_parts();

    let router = router
        .merge(Scalar::with_url("/docs", openapi))
        .layer(TraceLayer::new_for_http());

    // CORS is opt-in: absent in config means browser clients are not served.
    let router = match build_cors_layer(&config) {
        Some(cors) => router.layer(cors),
        None => router,
    };

    let router = router.with_state(states);
    serve(router, &config.addr).await
}

/// Builds the CORS layer from `cors.allow_origins` (`"*"` = any origin, a
/// comma-separated list = restricted origins). Returns `None` when CORS is
/// not configured.
fn build_cors_layer(config: &BackendConfig) -> Option<CorsLayer> {
    let origins = config
        .cors
        .allow_origins
        .as_deref()
        .map(|s| s.split(',').map(str::trim).filter(|s| !s.is_empty()))?
        .collect::<Vec<_>>();
    if origins.is_empty() {
        return None;
    }

    let layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);

    if origins.contains(&"*") {
        Some(layer.allow_origin(AllowOrigin::any()))
    } else {
        let origins = origins
            .into_iter()
            .filter_map(|o| HeaderValue::from_str(o).ok())
            .collect::<Vec<_>>();
        Some(layer.allow_origin(AllowOrigin::list(origins)))
    }
}

async fn serve(router: Router, addr: &str) -> Result<(), Error> {
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .inspect_err(|e| error!("failed to bind {addr}: {e}"))?;

    axum::serve(listener, router).await?;

    Ok(())
}
