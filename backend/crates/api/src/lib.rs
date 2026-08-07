//! SPEC-013: API layer — axum REST surface composing the platform specs.

pub mod auth;
pub mod docs;
pub mod error;
pub mod routes;

use utoipa::OpenApi;

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

/// OpenAPI document generated from the handler annotations.
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        routes::create_entity,
        routes::save_property_set,
        routes::list_property_sets,
        routes::resolve_chain,
        routes::submit_request,
        routes::decide_request,
        routes::apply_request,
        routes::audit_entity,
        routes::audit_actor,
        routes::list_namespaces,
        routes::validate_additional_type,
    ),
    components(schemas(
        routes::CreateEntityBody,
        routes::SavePropertySetBody,
        routes::ResolveQuery,
        routes::SubmitRequestBody,
        routes::DecideBody,
        routes::ValidateBody,
    ))
)]
pub struct ApiDoc;

/// Build the full API router (REST routes + mounted SSE stream + docs).
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
        .route("/openapi.json", get(openapi_json))
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/swagger")
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .with_state(state)
}

async fn openapi_json() -> axum::Json<utoipa::openapi::OpenApi> {
    axum::Json(ApiDoc::openapi())
}
