// SPEC-004 AC-4: SSE endpoint delivers events as commits land.

#![recursion_limit = "512"]

use std::time::Duration;

use futures::StreamExt;
use knowledge_domain::{EntityKind, PropertySet, Scope};
use terminusdb_bin::TerminusDBServer;
use terminusdb_repository::Repository;
use uuid::Uuid;

fn props(pairs: &[(&str, &str)]) -> std::collections::HashMap<String, serde_json::Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), serde_json::json!(v)))
        .collect()
}

#[tokio::test]
async fn sse_delivers_property_saved_event_within_poll_interval() -> anyhow::Result<()> {
    let server = TerminusDBServer::test_instance().await?;
    let client = server.client().await?;
    let db = format!("sse_{}", Uuid::now_v7().simple());
    let repo = Repository::new(client, db).await?;

    let app = stream_api::router(repo.clone(), Duration::from_millis(300));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Connect with a genesis cursor
    let url = format!("http://{addr}/events?cursor=");
    let resp = reqwest::get(&url).await?;
    let mut bytes = Box::pin(resp.bytes_stream());

    // Seed an entity BEFORE the client reads, then save a property set
    let entity = repo.create_entity(EntityKind::Node).await?;
    let org = Uuid::now_v7();
    // Note: entity creation already landed while the connection was open;
    // the property save is the event we assert on.
    repo.save_property_set(
        entity,
        org,
        &PropertySet::current(Scope::Org, 1, props(&[("name", "sse-probe")])),
        "alice",
        "sse-v1",
    )
    .await?;

    // Read the stream until the PropertySetSaved event appears
    let mut buf = String::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_secs(1), bytes.next()).await {
            Ok(Some(chunk)) => {
                buf.push_str(&String::from_utf8_lossy(&chunk?));
                if buf.contains("property_set_saved") {
                    break;
                }
            }
            _ => {}
        }
    }

    assert!(
        buf.contains("property_set_saved"),
        "property_set_saved event not received in stream: {buf}"
    );
    assert!(
        buf.contains(r#""scope":"Org""#),
        "event payload missing scope: {buf}"
    );
    assert!(
        buf.contains(r#""version":1"#),
        "event payload missing version: {buf}"
    );
    assert!(
        buf.contains("id: "),
        "SSE events must carry the advancing cursor id"
    );

    Ok(())
}
