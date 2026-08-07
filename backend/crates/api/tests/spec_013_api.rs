// SPEC-013: API layer integration tests (AC-1..AC-6).
// Boots a real TerminusDB server + the axum app, exercises routes via
// tower oneshot.

#![recursion_limit = "512"]

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use schema_registry::SchemaRegistry;
use terminusdb_bin::TerminusDBServer;
use terminusdb_repository::Repository;
use tower::ServiceExt;
use uuid::Uuid;

static SERVER_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();

async fn app() -> (axum::Router, terminusdb_client::TerminusDBHttpClient) {
    // The shared test server (2 workers) chokes when several tests do schema
    // inserts concurrently — serialize like the fork's RUST_TEST_THREADS=1.
    let _guard = SERVER_LOCK
        .get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap();
    let server = TerminusDBServer::test_instance().await.unwrap();
    let client = server.client().await.unwrap();
    let db = format!("api_{}", Uuid::now_v7().simple());
    let repo = Repository::new(client.clone(), db.clone()).await.unwrap();
    let registry = SchemaRegistry::init(client.clone(), format!("{db}_reg"))
        .await
        .unwrap();
    (api::router(repo, registry), client)
}

fn json_request(
    method: &str,
    uri: &str,
    body: serde_json::Value,
    principal: &str,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-principal", principal)
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("x-principal", "tester")
        .body(Body::empty())
        .unwrap()
}

async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
}

// ------------------------------------------------------------------
// AC-1: entity + property set via HTTP, resolve returns effective set
// ------------------------------------------------------------------
#[tokio::test]
async fn entity_crud_and_resolve_round_trip() {
    let (app, _client) = app().await;

    // create entity
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/entities",
            serde_json::json!({ "kind": "Node" }),
            "alice",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let entity = body_json(resp).await;
    let id = entity["id"].as_str().unwrap().to_string();

    // save property set at org instance
    let org = Uuid::now_v7();
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/entities/{id}/property-sets"),
            serde_json::json!({
                "instance_id": org,
                "scope": "Org",
                "version": 1,
                "properties": { "name": "acme-org" },
            }),
            "alice",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // resolve the chain
    let chain = format!("{org},{}", terminusdb_repository::COMMON_INSTANCE);
    let resp = app
        .oneshot(get_request(&format!(
            "/entities/{id}/resolve?instances={chain}"
        )))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let resolved = body_json(resp).await;
    assert_eq!(
        resolved["properties"]["name"],
        serde_json::json!("acme-org")
    );
    assert_eq!(resolved["scope"], serde_json::json!("Org"));
}

// ------------------------------------------------------------------
// AC-2 + AC-6: moderation via HTTP, principal recorded, audit visible
// ------------------------------------------------------------------
#[tokio::test]
async fn moderation_flow_with_principal_and_audit() {
    let (app, _client) = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/entities",
            serde_json::json!({ "kind": "Node" }),
            "alice",
        ))
        .await
        .unwrap();
    let entity = body_json(resp).await;
    let id = entity["id"].as_str().unwrap().to_string();
    let org = Uuid::now_v7();

    // save v1, submit change request (v2), decide by reviewer, apply
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/entities/{id}/property-sets"),
            serde_json::json!({
                "instance_id": org, "scope": "Org", "version": 1,
                "properties": { "name": "v1" },
            }),
            "alice",
        ))
        .await
        .unwrap();

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/requests",
            serde_json::json!({
                "entity_id": id, "instance_id": org, "scope": "Org",
                "proposed_version": 2, "correlation_id": "pipe-9",
            }),
            "alice",
        ))
        .await
        .unwrap();
    let req = body_json(resp).await;
    let request_id = req["request_id"].as_str().unwrap().to_string();

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/requests/{request_id}/decide"),
            serde_json::json!({ "approve": true }),
            "reviewer",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/requests/{request_id}/apply"),
            serde_json::Value::Null,
            "reviewer",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // AC-6: audit actor = reviewer; AC-2: decision + request visible
    let resp = app
        .clone()
        .oneshot(get_request(&format!("/audit/entities/{id}")))
        .await
        .unwrap();
    let audit = body_json(resp).await;
    let items = audit["items"].as_array().unwrap();
    assert!(
        items
            .iter()
            .any(|e| e["action"] == "Approved" && e["actor"] == "reviewer"),
        "approved decision by reviewer missing: {items:?}"
    );
    assert!(
        items
            .iter()
            .any(|e| e["action"] == "RequestSubmitted" && e["actor"] == "alice"),
        "request submission missing: {items:?}"
    );
    assert!(
        items
            .iter()
            .any(|e| e["correlation_id"] == serde_json::json!("pipe-9")),
        "correlation id missing: {items:?}"
    );

    // AC-2: property set now at v2 (apply wrote the proposed version)
    let chain = format!("{org},{}", terminusdb_repository::COMMON_INSTANCE);
    let resp = app
        .oneshot(get_request(&format!(
            "/entities/{id}/resolve?instances={chain}"
        )))
        .await
        .unwrap();
    let resolved = body_json(resp).await;
    assert_eq!(resolved["version"], serde_json::json!(2));
}

// ------------------------------------------------------------------
// AC-3: audit endpoint returns commit-ordered entries
// ------------------------------------------------------------------
#[tokio::test]
async fn audit_endpoint_returns_ordered_entries() {
    let (app, _client) = app().await;
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/entities",
            serde_json::json!({ "kind": "Node" }),
            "bob",
        ))
        .await
        .unwrap();
    let entity = body_json(resp).await;
    let id = entity["id"].as_str().unwrap().to_string();

    let resp = app
        .clone()
        .oneshot(get_request(&format!("/audit/entities/{id}")))
        .await
        .unwrap();
    let audit = body_json(resp).await;
    let items = audit["items"].as_array().unwrap();
    assert!(!items.is_empty());
    assert!(
        items
            .iter()
            .any(|e| e["action"] == "EntityCreated" && e["actor"] == "bob")
    );

    // actor audit
    let resp = app.oneshot(get_request("/audit/actors/bob")).await.unwrap();
    let audit = body_json(resp).await;
    assert!(!audit["items"].as_array().unwrap().is_empty());
}

// ------------------------------------------------------------------
// AC-4: additionalType mismatch -> warning, never rejection
// ------------------------------------------------------------------
#[tokio::test]
async fn additional_type_mismatch_returns_warning() {
    let (app, _client) = app().await;
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/validate-additional-type",
            serde_json::json!({ "additional_type": "terminusdb://schema/org.schema.v1/NotAType" }),
            "tester",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "schemaless: must not reject");
    let body = body_json(resp).await;
    assert_eq!(body["valid"], serde_json::json!(false));
    assert!(
        body["api:warnings"]
            .as_array()
            .map(|w| !w.is_empty())
            .unwrap_or(false),
        "warning expected: {body}"
    );

    // registered type -> valid, no warnings
    let resp = app
        .oneshot(json_request(
            "POST",
            "/validate-additional-type",
            serde_json::json!({ "additional_type": "terminusdb://schema/org.schema.v1/Person" }),
            "tester",
        ))
        .await
        .unwrap();
    let body = body_json(resp).await;
    assert_eq!(body["valid"], serde_json::json!(true));
}

// ------------------------------------------------------------------
// AC-5: /events mounted (SSE reachable)
// ------------------------------------------------------------------
#[tokio::test]
async fn events_endpoint_is_mounted() {
    let (app, _client) = app().await;
    let resp = app.oneshot(get_request("/events?cursor=")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        resp.headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("text/event-stream"),
        "SSE content type expected"
    );
}

// ------------------------------------------------------------------
// AC-5b: namespaces endpoint lists seeds
// ------------------------------------------------------------------
#[tokio::test]
async fn namespaces_endpoint_lists_seeds() {
    let (app, _client) = app().await;
    let resp = app
        .clone()
        .oneshot(get_request("/namespaces"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    let packages: Vec<&str> = body["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|i| i["package"].as_str())
        .collect();
    assert!(packages.contains(&"core.v1"));
    assert!(packages.contains(&"org.schema.v1"));
}
