// Tests for SPEC-004: Live Knowledge Stream (commit-cursor machinery).
// AC Coverage: AC-1, AC-2, AC-3, AC-5 (AC-4 in stream-api tests)

#![recursion_limit = "512"]

use data_graph::{EntityKind, PropertySet, Scope};
use terminusdb_bin::TerminusDBServer;
use terminusdb_repository::Repository;
use terminusdb_repository::stream::{CommitCursor, DomainEvent, poll_events};
use uuid::Uuid;

fn props(pairs: &[(&str, &str)]) -> std::collections::HashMap<String, serde_json::Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), serde_json::json!(v)))
        .collect()
}

async fn fresh_repo() -> anyhow::Result<(&'static TerminusDBServer, Repository)> {
    let server = TerminusDBServer::test_instance().await?;
    let client = server.client().await?;
    let db = format!("stream_{}", Uuid::now_v7().simple());
    let repo = Repository::new(client, db).await?;
    Ok((server, repo))
}

// ------------------------------------------------------------------
// AC-1: genesis cursor replays all events in commit order
// ------------------------------------------------------------------
#[tokio::test]
async fn genesis_cursor_replays_history_in_order() -> anyhow::Result<()> {
    let (_server, repo) = fresh_repo().await?;
    let entity = repo.create_entity(EntityKind::Node).await?;
    let org = Uuid::now_v7();
    repo.save_property_set(
        entity,
        org,
        &PropertySet::current(Scope::Org, 1, props(&[("name", "acme")])),
        "alice",
        "v1",
    )
    .await?;

    let (events, cursor) = poll_events(&repo, &CommitCursor::default()).await?;

    // EntityCreated (with its ent token), then PropertySetSaved
    assert_eq!(events.len(), 2, "expected 2 events: {events:?}");
    assert!(matches!(events[0], DomainEvent::EntityCreated { ref kind, .. } if kind == "Node"));
    assert!(matches!(
        events[1],
        DomainEvent::PropertySetSaved { version: 1, .. }
    ));
    assert!(!cursor.0.is_empty(), "cursor must advance past genesis");
    Ok(())
}

// ------------------------------------------------------------------
// AC-2: cursor at commit N -> only the new event, cursor advances
// ------------------------------------------------------------------
#[tokio::test]
async fn cursor_advances_and_returns_exactly_new_events() -> anyhow::Result<()> {
    let (_server, repo) = fresh_repo().await?;
    let entity = repo.create_entity(EntityKind::Node).await?;
    let org = Uuid::now_v7();
    repo.save_property_set(
        entity,
        org,
        &PropertySet::current(Scope::Org, 1, props(&[("name", "acme")])),
        "alice",
        "v1",
    )
    .await?;

    let (_events, cursor) = poll_events(&repo, &CommitCursor::default()).await?;

    // Nothing new yet
    let (events, cursor2) = poll_events(&repo, &cursor).await?;
    assert!(events.is_empty());
    assert_eq!(cursor2, cursor);

    // One save -> exactly one event
    repo.save_property_set(
        entity,
        org,
        &PropertySet::current(Scope::Org, 2, props(&[("name", "acme-v2")])),
        "bob",
        "v2",
    )
    .await?;
    let (events, cursor3) = poll_events(&repo, &cursor2).await?;
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        DomainEvent::PropertySetSaved { version: 2, .. }
    ));
    assert_ne!(cursor3, cursor2);
    Ok(())
}

// ------------------------------------------------------------------
// AC-3: multiple new commits -> each event once, in commit order
// ------------------------------------------------------------------
#[tokio::test]
async fn multiple_new_commits_emit_each_event_once_in_order() -> anyhow::Result<()> {
    let (_server, repo) = fresh_repo().await?;
    let entity = repo.create_entity(EntityKind::Node).await?;
    let org = Uuid::now_v7();
    let ws = Uuid::now_v7();
    repo.save_property_set(
        entity,
        org,
        &PropertySet::current(Scope::Org, 1, props(&[("name", "acme")])),
        "alice",
        "org-v1",
    )
    .await?;
    let (_, cursor) = poll_events(&repo, &CommitCursor::default()).await?;

    // Two more commits
    repo.save_property_set(
        entity,
        ws,
        &PropertySet::current(Scope::Workspace, 1, props(&[("name", "ws")])),
        "bob",
        "ws-v1",
    )
    .await?;
    repo.save_property_set(
        entity,
        org,
        &PropertySet::current(Scope::Org, 2, props(&[("name", "acme-v2")])),
        "bob",
        "org-v2",
    )
    .await?;

    let (events, _) = poll_events(&repo, &cursor).await?;
    assert_eq!(events.len(), 2, "each event exactly once: {events:?}");
    // chronological: ws save first, then org v2
    assert!(matches!(
        &events[0],
        DomainEvent::PropertySetSaved { instance_id, version: 1, .. }
            if *instance_id == ws
    ));
    assert!(matches!(
        &events[1],
        DomainEvent::PropertySetSaved { instance_id, version: 2, .. }
            if *instance_id == org
    ));
    Ok(())
}

// ------------------------------------------------------------------
// AC-5: unknown messages are skipped without failing the stream
// ------------------------------------------------------------------
#[tokio::test]
async fn unknown_commit_messages_are_skipped() -> anyhow::Result<()> {
    let (_server, repo) = fresh_repo().await?;

    // Insert a document with an unrelated message to simulate an unknown commit
    let doc = terminusdb_repository::EntityDoc {
        id: terminusdb_schema::EntityIDFor::new("E:garbage-unknown")?,
        kind: "Node".to_string(),
    };
    let mut args =
        terminusdb_client::DocumentInsertArgs::from(terminusdb_client::BranchSpec::from(repo.db()));
    args.message = "some unrelated commit".to_string();
    repo.client().insert(&doc, args).await?;

    let (events, _) = poll_events(&repo, &CommitCursor::default()).await?;
    assert!(
        events.is_empty(),
        "unknown messages must not produce events: {events:?}"
    );
    Ok(())
}
