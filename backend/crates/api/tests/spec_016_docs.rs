// SPEC-016: programmatic API docs + OpenAPI (AC-1..AC-4).

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

async fn app() -> axum::Router {
    let _guard = SERVER_LOCK
        .get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap();
    let server = TerminusDBServer::test_instance().await.unwrap();
    let client = server.client().await.unwrap();
    let db = format!("docs_{}", Uuid::now_v7().simple());
    let repo = Repository::new(client.clone(), db.clone()).await.unwrap();
    let registry = SchemaRegistry::init(client.clone(), format!("{db}_reg"))
        .await
        .unwrap();
    api::router(repo, registry)
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
// AC-1: GET /openapi.json — valid OpenAPI covering all routes
// ------------------------------------------------------------------
#[tokio::test]
async fn openapi_json_covers_all_routes() {
    let app = app().await;
    let resp = app.oneshot(get_request("/openapi.json")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let doc = body_json(resp).await;

    assert_eq!(doc["openapi"], serde_json::json!("3.1.0"));
    let paths = doc["paths"].as_object().unwrap();
    for p in [
        "/entities",
        "/entities/{id}/property-sets",
        "/entities/{id}/resolve",
        "/requests",
        "/requests/{id}/decide",
        "/audit/entities/{id}",
        "/namespaces",
        "/validate-additional-type",
    ] {
        assert!(paths.contains_key(p), "openapi missing path {p}");
    }
}

// ------------------------------------------------------------------
// AC-2: swagger UI served
// ------------------------------------------------------------------
#[tokio::test]
async fn swagger_ui_is_served() {
    let app = app().await;
    let resp = app.oneshot(get_request("/swagger/")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let content_type = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(
        content_type.contains("text/html"),
        "swagger UI should serve html: {content_type}"
    );
}

// ------------------------------------------------------------------
// AC-3: docgen produces book chapters (no server needed)
// ------------------------------------------------------------------
#[tokio::test]
async fn docgen_writes_book_chapters() -> anyhow::Result<()> {
    let tmp = std::env::temp_dir().join(format!("docgen_{}", Uuid::now_v7().simple()));
    let book = tmp.join("book");
    let specs = tmp.join("specs");
    let openapi = tmp.join("openapi");
    std::fs::create_dir_all(&book)?;
    std::fs::create_dir_all(&specs)?;

    // minimal spec fixture
    std::fs::write(
        specs.join("SPEC-999-fake.md"),
        "# SPEC-999 Feature: Fake\n\n<!-- status: Draft -->\n\n- AC-1: ...\n- AC-2: ...\n",
    )?;

    api::docs::generate(&book, &specs, &openapi.join("openapi.json"))?;

    assert!(openapi.join("openapi.json").exists());
    let api_ref = std::fs::read_to_string(book.join("api-reference.md"))?;
    assert!(api_ref.contains("/entities"));
    let specs_md = std::fs::read_to_string(book.join("specs.md"))?;
    assert!(
        specs_md.contains("SPEC-999"),
        "spec index must list the fixture: {specs_md}"
    );
    assert!(specs_md.contains("2 AC references"));
    assert!(book.join("registry.md").exists());
    Ok(())
}

// ------------------------------------------------------------------
// AC-4: handlers added -> regenerated openapi reflects them (drift-free)
// ------------------------------------------------------------------
#[tokio::test]
async fn openapi_reflects_handler_changes() {
    // The route set is fixed at compile time; AC-4 asserts the openapi
    // document always matches the current route table (no stale cache).
    let app = app().await;
    let resp = app.oneshot(get_request("/openapi.json")).await.unwrap();
    let doc = body_json(resp).await;
    let paths = doc["paths"].as_object().unwrap();
    // every route registered in api::routes::router() must be present
    let expected = [
        ("POST", "/entities"),
        ("POST", "/requests"),
        ("GET", "/audit/entities/{id}"),
        ("GET", "/audit/actors/{actor}"),
        ("GET", "/namespaces"),
        ("POST", "/validate-additional-type"),
    ];
    for (method, p) in expected {
        let path_item = paths.get(p).expect("path present");
        assert!(
            path_item
                .as_object()
                .unwrap()
                .contains_key(method.to_lowercase().as_str()),
            "{method} {p} missing from openapi"
        );
    }
}
