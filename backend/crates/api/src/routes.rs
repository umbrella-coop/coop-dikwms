//! REST routes: entities, property sets, moderation, audit, registry.

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use data_graph::{PropertySet, Scope, Status};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::auth::Principal;
use crate::error::ApiError;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/entities", post(create_entity))
        .route("/bulk-entities", post(bulk_entities))
        .route("/entities/{id}/property-sets", post(save_property_set))
        .route("/entities/{id}/property-sets", get(list_property_sets))
        .route("/entities/{id}/resolve", get(resolve_chain))
        .route("/requests", post(submit_request))
        .route("/requests/{id}/decide", post(decide_request))
        .route("/requests/{id}/apply", post(apply_request))
        .route("/audit/entities/{id}", get(audit_entity))
        .route("/audit/actors/{actor}", get(audit_actor))
        .route("/namespaces", get(list_namespaces))
        .route("/validate-additional-type", post(validate_additional_type))
}

// ------------------------------------------------------------------
// Entities & property sets
// ------------------------------------------------------------------

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateEntityBody {
    kind: String,
}

#[utoipa::path(post, path = "/entities")]
pub async fn create_entity(
    State(state): State<AppState>,
    Principal(principal): Principal,
    Json(body): Json<CreateEntityBody>,
) -> Result<Json<Value>, ApiError> {
    let kind = match body.kind.as_str() {
        "Node" => data_graph::EntityKind::Node,
        "Edge" => data_graph::EntityKind::Edge,
        "Combo" => data_graph::EntityKind::Combo,
        other => return Err(ApiError::bad_request(format!("unknown kind: {other}"))),
    };
    let id = state.repo.create_entity_as(kind, &principal).await?;
    tracing::info!(entity = %id, principal = %principal, "entity created");
    Ok(Json(json!({ "@type": "api:Entity", "id": id })))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SavePropertySetBody {
    instance_id: Uuid,
    scope: String,
    version: u64,
    properties: Value,
    correlation_id: Option<String>,
}

#[utoipa::path(post, path = "/entities/{id}/property-sets")]
pub async fn save_property_set(
    State(state): State<AppState>,
    Principal(principal): Principal,
    Path(id): Path<Uuid>,
    Json(body): Json<SavePropertySetBody>,
) -> Result<Json<Value>, ApiError> {
    let scope = match body.scope.as_str() {
        "Common" => Scope::Common,
        "Org" => Scope::Org,
        "Workspace" => Scope::Workspace,
        "Project" => Scope::Project,
        other => return Err(ApiError::bad_request(format!("unknown scope: {other}"))),
    };
    let properties: std::collections::HashMap<String, Value> = body
        .properties
        .as_object()
        .map(|m| m.clone().into_iter().collect())
        .unwrap_or_default();
    let set = PropertySet {
        scope,
        version: body.version,
        status: Status::Current,
        properties,
    };
    state
        .repo
        .save_property_set_correlated(
            id,
            body.instance_id,
            &set,
            &principal,
            "api-write",
            body.correlation_id.as_deref(),
        )
        .await?;
    Ok(Json(
        json!({ "@type": "api:Ok", "entity": id, "instance": body.instance_id, "version": body.version }),
    ))
}

// ------------------------------------------------------------------
// Bulk ingestion (SPEC-027 REQ-006)
// ------------------------------------------------------------------

#[derive(Deserialize)]
pub struct BulkEntity {
    kind: data_graph::EntityKind,
    /// Value of the dedupe key (e.g. commit hexsha, normalized email) —
    /// server-side idempotency: entities whose property set already carries
    /// `dedupe_key` = this value are skipped, not duplicated.
    idempotency_key: String,
    /// data-graph domain model as the wire contract: scope/version/status/
    /// properties arrive typed, no manual string mapping.
    set: data_graph::PropertySet,
}

#[derive(Deserialize)]
pub struct BulkEntitiesBody {
    /// Property key to dedupe on ("hash" | "email").
    dedupe_key: String,
    entities: Vec<BulkEntity>,
}

/// Batch create entities + property sets with server-side idempotency
/// (SPEC-027 REQ-006; dev-only — RISK-001). OpenAPI schema deferred.
pub async fn bulk_entities(
    State(state): State<AppState>,
    Principal(principal): Principal,
    Json(body): Json<BulkEntitiesBody>,
) -> Result<Json<Value>, ApiError> {
    let mut results = Vec::with_capacity(body.entities.len());
    let mut warnings: Vec<Value> = Vec::new();

    let existing = state.repo.property_index(&body.dedupe_key).await?;

    for entity in body.entities {
        // Internal insight entities (git-insight-*) carry non-git properties —
        // excluded from the git.v1 lint to keep organic warnings honest.
        if !entity.idempotency_key.starts_with("git-insight-") {
            warnings.extend(git_v1_warnings(&state, &entity.set).await);
        }
        if let Some(existing_ids) = existing.get(&entity.idempotency_key) {
            results.push(bulk_result(
                &entity,
                existing_ids.first().copied(),
                "skipped",
                "already imported",
            ));
            continue;
        }
        let entity_id = match state.repo.create_entity_as(entity.kind, &principal).await {
            Ok(id) => id,
            Err(e) => {
                results.push(bulk_result(&entity, None, "skipped", &format!("{e:#}")));
                continue;
            }
        };
        match state
            .repo
            .save_property_set(
                entity_id,
                terminusdb_repository::COMMON_INSTANCE,
                &entity.set,
                &principal,
                "bulk-import",
            )
            .await
        {
            Ok(()) => {
                results.push(bulk_result(&entity, Some(entity_id), "imported", ""));
            }
            Err(e) => {
                results.push(bulk_result(&entity, None, "skipped", &format!("{e:#}")));
            }
        }
    }

    Ok(Json(json!({
        "@type": "api:BulkResult",
        "results": results,
        "warnings": warnings,
    })))
}

fn bulk_result(entity: &BulkEntity, entity_id: Option<Uuid>, status: &str, reason: &str) -> Value {
    let mut v = json!({
        "idempotency_key": entity.idempotency_key,
        "status": status,
    });
    if let Some(id) = entity_id {
        v["entity_id"] = json!(id);
    }
    if !reason.is_empty() {
        v["reason"] = json!(reason);
    }
    v
}

/// git.v1 schemaless lint (SPEC-027 REQ-007, AC-7): compare entity properties
/// against the registered `git.v1` message fields — warn, never reject.
/// Organic real-data violations (unknown/extra properties, empty messages)
/// surface as `api:warnings`.
async fn git_v1_warnings(state: &AppState, set: &data_graph::PropertySet) -> Vec<Value> {
    let Ok(Some(doc)) = state.registry.latest_for_package("git.v1").await else {
        return Vec::new();
    };
    let Ok(fds) = doc.descriptor() else {
        return Vec::new();
    };
    let message_name = if set.properties.contains_key("hash") {
        "Commit"
    } else if set.properties.contains_key("email") {
        "Author"
    } else {
        return Vec::new();
    };
    let Some(message) = schema_registry::descriptor::message(&fds, message_name) else {
        return Vec::new();
    };
    let known: std::collections::HashSet<String> = schema_registry::descriptor::field_tags(message)
        .iter()
        .map(|(name, _)| name.clone())
        .collect();
    let mut warnings = Vec::new();
    for key in set.properties.keys() {
        if !known.contains(key) {
            warnings.push(json!({
                "@type": "api:Warning",
                "message": format!("git.v1: unknown property {key} on {message_name}"),
                "property": key,
            }));
        }
    }
    if message_name == "Commit"
        && set
            .properties
            .get("message")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .is_some_and(str::is_empty)
    {
        warnings.push(json!({
            "@type": "api:Warning",
            "message": "git.v1: Commit.message is empty",
        }));
    }
    warnings
}

#[utoipa::path(get, path = "/entities/{id}/property-sets")]
pub async fn list_property_sets(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let docs = state.repo.load_property_sets(id).await?;
    let out: Vec<Value> = docs
        .iter()
        .map(|d| {
            json!({
                "instance_id": d.instance_id,
                "scope": d.scope,
                "version": d.version,
                "status": d.status,
                "properties": d.properties,
            })
        })
        .collect();
    Ok(Json(json!({ "@type": "api:PropertySets", "items": out })))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ResolveQuery {
    instances: String,
}

#[utoipa::path(get, path = "/entities/{id}/resolve")]
pub async fn resolve_chain(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<ResolveQuery>,
) -> Result<Json<Value>, ApiError> {
    let chain: Vec<Uuid> = q
        .instances
        .split(',')
        .map(|s| s.trim().parse())
        .collect::<Result<_, _>>()
        .map_err(|_| ApiError::bad_request("instances must be comma-separated uuids"))?;
    if chain.is_empty() {
        return Err(ApiError::bad_request("instances required"));
    }
    let head = state.repo.latest_commit().await?;
    let set = terminusdb_repository::resolve_at(&state.repo, id, &chain, &head)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(json!({
        "@type": "api:ResolvedPropertySet",
        "scope": format!("{:?}", set.scope),
        "version": set.version,
        "status": format!("{:?}", set.status),
        "properties": set.properties,
    })))
}

// ------------------------------------------------------------------
// Moderation (thin wrapper over ledger docs + repository)
// ------------------------------------------------------------------

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SubmitRequestBody {
    entity_id: Uuid,
    instance_id: Uuid,
    scope: String,
    proposed_version: u64,
    correlation_id: Option<String>,
}

#[utoipa::path(post, path = "/requests")]
pub async fn submit_request(
    State(state): State<AppState>,
    Principal(principal): Principal,
    Json(body): Json<SubmitRequestBody>,
) -> Result<Json<Value>, ApiError> {
    let request_id = state
        .repo
        .save_change_request(
            body.entity_id,
            body.instance_id,
            &body.scope,
            body.proposed_version,
            &principal,
            body.correlation_id.as_deref(),
        )
        .await?;
    Ok(Json(
        json!({ "@type": "api:ChangeRequest", "request_id": request_id }),
    ))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DecideBody {
    approve: bool,
}

#[utoipa::path(post, path = "/requests/{id}/decide")]
pub async fn decide_request(
    State(state): State<AppState>,
    Principal(principal): Principal,
    Path(request_id): Path<Uuid>,
    Json(body): Json<DecideBody>,
) -> Result<Json<Value>, ApiError> {
    let requests = state.repo.requests_for_entity_any().await?;
    let req = requests
        .iter()
        .find(|(r, _)| r.request_id == request_id.to_string())
        .ok_or_else(|| ApiError::not_found(format!("request {request_id}")))?;
    let entity: Uuid = req
        .0
        .entity_id
        .parse()
        .map_err(|_| ApiError::internal("bad entity id in request doc"))?;
    state
        .repo
        .save_decision(request_id, entity, &principal, body.approve)
        .await?;
    Ok(Json(
        json!({ "@type": "api:Decision", "request_id": request_id, "approve": body.approve, "decided_by": principal }),
    ))
}

#[utoipa::path(post, path = "/requests/{id}/apply")]
pub async fn apply_request(
    State(state): State<AppState>,
    Principal(principal): Principal,
    Path(request_id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let requests = state.repo.requests_for_entity_any().await?;
    let req = requests
        .iter()
        .find(|(r, _)| r.request_id == request_id.to_string())
        .ok_or_else(|| ApiError::not_found(format!("request {request_id}")))?;
    let entity: Uuid = req
        .0
        .entity_id
        .parse()
        .map_err(|_| ApiError::internal("bad entity id in request doc"))?;
    let instance: Uuid = req
        .0
        .instance_id
        .parse()
        .map_err(|_| ApiError::internal("bad instance id in request doc"))?;
    let scope = match req.0.scope.as_str() {
        "Common" => Scope::Common,
        "Org" => Scope::Org,
        "Workspace" => Scope::Workspace,
        "Project" => Scope::Project,
        _ => return Err(ApiError::internal("bad scope in request doc")),
    };
    // Apply = write the proposed version (thin v1 semantics; the ledger state
    // machine adoption is documented in SPEC-013 R-19).
    let set = PropertySet {
        scope,
        version: req.0.proposed_version as u64,
        status: Status::Current,
        properties: Default::default(),
    };
    state
        .repo
        .save_property_set(
            entity,
            instance,
            &set,
            &principal,
            &format!("apply {request_id}"),
        )
        .await?;
    Ok(Json(
        json!({ "@type": "api:ApplyResult", "request_id": request_id, "applied": true }),
    ))
}

// ------------------------------------------------------------------
// Audit
// ------------------------------------------------------------------

#[utoipa::path(get, path = "/audit/entities/{id}")]
pub async fn audit_entity(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let entries = state.repo.audit_for_entity(id).await?;
    let out: Vec<Value> = entries
        .iter()
        .map(|e| {
            json!({
                "commit": e.commit,
                "actor": e.actor,
                "action": format!("{:?}", e.action),
                "entity_id": e.entity_id,
                "correlation_id": e.correlation_id,
                "details": e.details,
            })
        })
        .collect();
    Ok(Json(json!({ "@type": "api:AuditEntries", "items": out })))
}

#[utoipa::path(get, path = "/audit/actors/{actor}")]
pub async fn audit_actor(
    State(state): State<AppState>,
    Path(actor): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let entries = state.repo.audit_for_actor(&actor).await?;
    let out: Vec<Value> = entries
        .iter()
        .map(|e| {
            json!({
                "commit": e.commit,
                "actor": e.actor,
                "action": format!("{:?}", e.action),
                "entity_id": e.entity_id,
                "correlation_id": e.correlation_id,
            })
        })
        .collect();
    Ok(Json(json!({ "@type": "api:AuditEntries", "items": out })))
}

// ------------------------------------------------------------------
// Registry (schemaless philosophy: warnings, never rejections)
// ------------------------------------------------------------------

#[utoipa::path(get, path = "/namespaces")]
pub async fn list_namespaces(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let docs = state.registry.list_namespaces().await?;
    let out: Vec<Value> = docs
        .iter()
        .map(|d| {
            json!({
                "package": d.package,
                "version": d.version,
                "types": d.types,
            })
        })
        .collect();
    Ok(Json(json!({ "@type": "api:Namespaces", "items": out })))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ValidateBody {
    additional_type: String,
}

#[utoipa::path(post, path = "/validate-additional-type")]
pub async fn validate_additional_type(
    State(state): State<AppState>,
    Json(body): Json<ValidateBody>,
) -> Result<Json<Value>, ApiError> {
    // Schemaless philosophy: mismatch -> warning in the response, never reject.
    let (valid, warnings) = match state
        .registry
        .validate_additional_type(&body.additional_type)
        .await
    {
        Ok(()) => (true, Vec::new()),
        Err(schema_registry::RegistryError::TypeNotFound(t)) => (
            false,
            vec![
                json!({ "@type": "api:Warning", "api:message": format!("type not registered: {t}") }),
            ],
        ),
        Err(schema_registry::RegistryError::NotFound(p)) => (
            false,
            vec![
                json!({ "@type": "api:Warning", "api:message": format!("namespace not registered: {p}") }),
            ],
        ),
        Err(e) => return Err(ApiError::bad_request(e.to_string())),
    };
    Ok(Json(json!({
        "@type": "api:AdditionalTypeValidation",
        "valid": valid,
        "api:warnings": warnings,
    })))
}
