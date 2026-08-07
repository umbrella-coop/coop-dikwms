//! SPEC-013: API layer — axum REST surface composing the platform specs.

pub mod auth;
pub mod error;
pub mod routes;

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::FromRef;
use axum::routing::get;
use schema_registry::SchemaRegistry;
use stream_api::StreamState;
use terminusdb_repository::Repository;

pub const POLL_INTERVAL: Duration = Duration::from_millis(300);

#[derive(Clone, FromRef)]
pub struct AppState {
    pub repo: Arc<Repository>,
    pub registry: Arc<SchemaRegistry>,
    pub stream: StreamState,
}

/// Build the full API router (REST routes + mounted SSE stream).
pub fn router(repo: Repository, registry: SchemaRegistry) -> Router {
    let repo = Arc::new(repo);
    let state = AppState {
        repo: repo.clone(),
        registry: Arc::new(registry),
        stream: StreamState {
            repo,
            poll_interval: POLL_INTERVAL,
        },
    };
    routes::router()
        .route("/events", get(stream_api::events_handler))
        .with_state(state)
}
