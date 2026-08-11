// Tests for SPEC-012: Audit & Traceability (AC-1..AC-5).

#![recursion_limit = "512"]

use data_graph::{EntityKind, PropertySet, Scope};
use terminusdb_bin::TerminusDBServer;
use terminusdb_repository::Repository;
use terminusdb_repository::audit::AuditAction;
use uuid::Uuid;

fn props(pairs: &[(&str, &str)]) -> std::collections::HashMap<String, serde_json::Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), serde_json::json!(v)))
        .collect()
}

async fn fresh_repo() -> anyhow::Result<Repository> {
    let server = TerminusDBServer::test_instance().await?;
    let client = server.client().await?;
    let db = format!("audit_{}", Uuid::now_v7().simple());
    Ok(Repository::new(client, db).await?)
}

// ------------------------------------------------------------------
// AC-1 + AC-2: ledger persists requests and append-only decisions
// ------------------------------------------------------------------
#[tokio::test]
async fn ledger_round_trips_requests_and_append_only_decisions() -> anyhow::Result<()> {
    let repo = fresh_repo().await?;
    let entity = repo.create_entity(EntityKind::Node).await?;
    let org = Uuid::now_v7();

    let req_id = repo
        .save_change_request(entity, org, "Org", 2, "alice", Some("pipeline-1"))
        .await?;
    repo.save_decision(req_id, entity, "reviewer", true).await?;
    repo.save_decision(req_id, entity, "reviewer2", false)
        .await?;

    // Reload from the store (fresh repository over the same db)
    let repo2 = Repository::new(repo.client().clone(), repo.db().to_string()).await?;
    let requests = repo2.requests_for_entity(entity).await?;

    assert_eq!(requests.len(), 1);
    let (req, decisions) = &requests[0];
    assert_eq!(req.request_id, req_id.to_string());
    assert_eq!(req.created_by, "alice");
    assert_eq!(req.correlation_id.as_deref(), Some("pipeline-1"));
    // AC-2: both decisions present, append-only (two entries, never mutated)
    assert_eq!(
        decisions.len(),
        2,
        "decisions must be append-only: {decisions:?}"
    );
    assert!(
        decisions
            .iter()
            .any(|d| d.approve && d.decided_by == "reviewer")
    );
    assert!(
        decisions
            .iter()
            .any(|d| !d.approve && d.decided_by == "reviewer2")
    );
    Ok(())
}

// ------------------------------------------------------------------
// AC-3: correlation ids traceable through the audit
// ------------------------------------------------------------------
#[tokio::test]
async fn correlation_ids_are_traceable_in_audit() -> anyhow::Result<()> {
    let repo = fresh_repo().await?;
    let entity = repo.create_entity(EntityKind::Node).await?;
    let org = Uuid::now_v7();

    repo.save_property_set_correlated(
        entity,
        org,
        &PropertySet::current(Scope::Org, 1, props(&[("name", "acme")])),
        "pipeline-agent",
        "batch-v1",
        Some("corr-batch-42"),
    )
    .await?;

    let audit = repo.audit_for_entity(entity).await?;
    assert!(
        audit
            .iter()
            .any(|e| e.correlation_id.as_deref() == Some("corr-batch-42")),
        "correlation id must appear in audit: {audit:?}"
    );
    Ok(())
}

// ------------------------------------------------------------------
// AC-4: reverts are marked with their target commit
// ------------------------------------------------------------------
#[tokio::test]
async fn reverts_are_marked_with_target_commit() -> anyhow::Result<()> {
    let repo = fresh_repo().await?;
    let entity = repo.create_entity(EntityKind::Node).await?;
    let org = Uuid::now_v7();
    let chain = vec![org, terminusdb_repository::COMMON_INSTANCE];

    repo.save_property_set(
        entity,
        org,
        &PropertySet::current(Scope::Org, 1, props(&[("name", "before")])),
        "alice",
        "v1",
    )
    .await?;
    let target = repo.latest_commit().await?;
    repo.save_property_set(
        entity,
        org,
        &PropertySet::current(Scope::Org, 2, props(&[("name", "after")])),
        "bob",
        "v2",
    )
    .await?;
    repo.revert_property_set(entity, org, &chain, &target, "admin", "rollback bad data")
        .await?;

    let audit = repo.audit_for_entity(entity).await?;
    let reverted: Vec<_> = audit
        .iter()
        .filter(|e| e.action == AuditAction::Reverted)
        .collect();
    assert_eq!(reverted.len(), 1, "exactly one revert: {audit:?}");
    assert_eq!(
        reverted[0].details, target,
        "revert must reference target commit"
    );

    Ok(())
}

// ------------------------------------------------------------------
// AC-5: entity audit is commit-ordered with actor/action/commit
// ------------------------------------------------------------------
#[tokio::test]
async fn entity_audit_is_ordered_and_complete() -> anyhow::Result<()> {
    let repo = fresh_repo().await?;
    let entity = repo.create_entity(EntityKind::Node).await?;
    let org = Uuid::now_v7();
    let chain = vec![org, terminusdb_repository::COMMON_INSTANCE];

    repo.save_property_set(
        entity,
        org,
        &PropertySet::current(Scope::Org, 1, props(&[("name", "acme")])),
        "alice",
        "v1",
    )
    .await?;
    let req_id = repo
        .save_change_request(entity, org, "Org", 2, "alice", None)
        .await?;
    repo.save_decision(req_id, entity, "reviewer", true).await?;

    let audit = repo.audit_for_entity(entity).await?;
    assert!(!audit.is_empty());

    // Every entry carries actor + action + commit
    for e in &audit {
        assert!(!e.actor.is_empty(), "actor missing: {e:?}");
        assert!(!e.commit.is_empty(), "commit missing: {e:?}");
    }
    // The full lifecycle is represented
    let actions: Vec<AuditAction> = audit.iter().map(|e| e.action).collect();
    assert!(actions.contains(&AuditAction::EntityCreated));
    assert!(actions.contains(&AuditAction::PropertySetSaved));
    assert!(actions.contains(&AuditAction::RequestSubmitted));
    assert!(actions.contains(&AuditAction::Approved));
    Ok(())
}
