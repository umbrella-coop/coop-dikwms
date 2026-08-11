// Tests for SPEC-006: TerminusDB as the system of record (TDD).
// All tests run against a real per-process TerminusDB 12.1 server
// (terminusdb_bin::TerminusDBServer).
// AC Coverage: AC-1, AC-2, AC-3, AC-6 (AC-4/AC-5 covered by fork tests + v2 branch mapping)

#![recursion_limit = "512"]

use data_graph::{EntityKind, PropertySet, Scope};
use terminusdb_bin::TerminusDBServer;
use terminusdb_repository::{Repository, resolve_at};
use uuid::Uuid;

fn props(pairs: &[(&str, &str)]) -> std::collections::HashMap<String, serde_json::Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), serde_json::json!(v)))
        .collect()
}

// ------------------------------------------------------------------
// AC-1 + AC-6: Entity and scoped property sets survive a re-load with
// versions intact; ladder resolution invariants hold after reload.
// ------------------------------------------------------------------
#[tokio::test]
async fn entity_and_property_sets_round_trip_with_versions_intact() -> anyhow::Result<()> {
    let server = TerminusDBServer::test_instance().await?;
    let client = server.client().await?;
    let db = format!("repo_ac1_{}", Uuid::now_v7().simple());
    let repo = Repository::new(client, db).await?;

    // Persist an entity with an org-level property set (v1)
    let entity_id = repo.create_entity(EntityKind::Node).await?;
    let org = Uuid::now_v7();
    let set_v1 = PropertySet::current(Scope::Org, 1, props(&[("name", "acme-org")]));
    repo.save_property_set(entity_id, org, &set_v1, "alice", "request-1")
        .await?;

    // Reload from the store (fresh repository over the same database)
    let repo2 = Repository::new(repo.client().clone(), repo.db().to_string()).await?;
    let loaded = repo2.load_property_sets(entity_id).await?;

    // AC-1: entity + property set present, versions intact
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].version, 1);
    assert_eq!(loaded[0].instance_id, org.to_string());
    assert_eq!(loaded[0].properties["name"], serde_json::json!("acme-org"));

    // AC-6: the loaded document converts back to a domain property set
    let set = loaded[0].to_property_set();
    assert_eq!(set.version, 1);
    assert_eq!(set.scope, Scope::Org);
    assert_eq!(set.status, data_graph::Status::Current);

    Ok(())
}

// ------------------------------------------------------------------
// AC-2: Saving a property set creates a commit with author + message
// ------------------------------------------------------------------
#[tokio::test]
async fn property_set_save_commits_with_author_and_message() -> anyhow::Result<()> {
    let server = TerminusDBServer::test_instance().await?;
    let client = server.client().await?;
    let db = format!("repo_ac2_{}", Uuid::now_v7().simple());
    let repo = Repository::new(client, db).await?;

    let entity_id = repo.create_entity(EntityKind::Node).await?;
    let org = Uuid::now_v7();
    let set = PropertySet::current(Scope::Org, 1, Default::default());
    repo.save_property_set(entity_id, org, &set, "reviewer", "request-42")
        .await?;

    // The commit log must carry author = reviewer, message = request id
    let log = repo.commit_log_for_entity(entity_id).await?;
    let entry = log
        .iter()
        .find(|e| e.message.starts_with("request-42"))
        .expect("commit with message request-42 not found");
    assert_eq!(entry.author, "reviewer");

    Ok(())
}

// ------------------------------------------------------------------
// AC-3: As-of resolution returns the pre-change property set
// ------------------------------------------------------------------
#[tokio::test]
async fn resolve_at_returns_state_before_change_commit() -> anyhow::Result<()> {
    let server = TerminusDBServer::test_instance().await?;
    let client = server.client().await?;
    let db = format!("repo_ac3_{}", Uuid::now_v7().simple());
    let repo = Repository::new(client, db).await?;

    let entity_id = repo.create_entity(EntityKind::Node).await?;
    let org = Uuid::now_v7();
    let chain = vec![org, terminusdb_repository::COMMON_INSTANCE];

    let set_v1 = PropertySet::current(Scope::Org, 1, props(&[("name", "before")]));
    repo.save_property_set(entity_id, org, &set_v1, "alice", "v1")
        .await?;
    let commit_v1 = repo.latest_commit().await?;

    let set_v2 = PropertySet::current(Scope::Org, 2, props(&[("name", "after")]));
    repo.save_property_set(entity_id, org, &set_v2, "bob", "v2")
        .await?;

    // As of the first commit, the name must still be "before"
    let as_of = resolve_at(&repo, entity_id, &chain, &commit_v1).await?;
    assert_eq!(as_of.properties["name"], serde_json::json!("before"));

    // Current resolution (no as-of) must see "after"
    let current = resolve_at(&repo, entity_id, &chain, &repo.latest_commit().await?).await?;
    assert_eq!(current.properties["name"], serde_json::json!("after"));

    Ok(())
}
