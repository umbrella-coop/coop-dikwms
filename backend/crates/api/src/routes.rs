//! REST routes: entities, property sets, moderation, audit, registry.

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use knowledge_domain::{PropertySet, Scope, Status};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::auth::Principal;
use crate::error::ApiError;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/entities", post(create_entity))
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

#[derive(Deserialize)]
struct CreateEntityBody {
    kind: String,
}

async fn create_entity(
    State(state): State<AppState>,
    Principal(principal): Principal,
    Json(body): Json<CreateEntityBody>,
) -> Result<Json<Value>, ApiError> {
    let kind = match body.kind.as_str() {
        "Node" => knowledge_domain::EntityKind::Node,
        "Edge" => knowledge_domain::EntityKind::Edge,
        "Combo" => knowledge_domain::EntityKind::Combo,
        other => return Err(ApiError::bad_request(format!("unknown kind: {other}"))),
    };
    let id = state.repo.create_entity_as(kind, &principal).await?;
    tracing::info!(entity = %id, principal = %principal, "entity created");
    Ok(Json(json!({ "@type": "api:Entity", "id": id })))
}

#[derive(Deserialize)]
struct SavePropertySetBody {
    instance_id: Uuid,
    scope: String,
    version: u64,
    properties: Value,
    correlation_id: Option<String>,
}

async fn save_property_set(
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

async fn list_property_sets(
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

#[derive(Deserialize)]
struct ResolveQuery {
    instances: String,
}

async fn resolve_chain(
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

#[derive(Deserialize)]
struct SubmitRequestBody {
    entity_id: Uuid,
    instance_id: Uuid,
    scope: String,
    proposed_version: u64,
    correlation_id: Option<String>,
}

async fn submit_request(
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

#[derive(Deserialize)]
struct DecideBody {
    approve: bool,
}

async fn decide_request(
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

async fn apply_request(
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

async fn audit_entity(
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

async fn audit_actor(
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

async fn list_namespaces(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
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

#[derive(Deserialize)]
struct ValidateBody {
    additional_type: String,
}

async fn validate_additional_type(
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
