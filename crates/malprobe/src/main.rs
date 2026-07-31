use axum::Router;
use log::error;
use malprobe_common::error::Error;
use malprobe_config::BackendConfig;
use mimalloc::MiMalloc;
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

        AppState {
            database,
            _unit: (),
        }
    };

    let (router, openapi) = OpenApiRouter::new()
        .merge(endpoints::router())
        .split_for_parts();

    let router: Router = router
        .merge(Scalar::with_url("/docs", openapi))
        .layer(TraceLayer::new_for_http())
        .with_state(states);

    let listener = tokio::net::TcpListener::bind(config.addr)
        .await
        .inspect_err(|e| error!("{e}"))?;

    axum::serve(listener, router).await?;

    Ok(())
}
