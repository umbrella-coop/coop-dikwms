// Tests for SPEC-002: Moderation & Promotion Workflow
// Derived from: docs/specs/SPEC-002-moderation-promotion.md (Approved 2026-08-06)
// TDD: RED phase — these tests target the new public API of `knowledge-domain`.
// AC Coverage: AC-1..AC-6 (1:1 mapping)

use std::collections::HashMap;

use knowledge_domain::{
    ApplyOutcome, ChangeRequestStatus, EntityKind, Graph, ModerationLedger, PropertySet, Scope,
};
use serde_json::json;

// ------------------------------------------------------------------
// AC-1: Submission creates a request without changing current truth
// ------------------------------------------------------------------
#[test]
fn submitting_change_request_does_not_change_current_truth() {
    // Arrange
    let mut graph = Graph::new();
    let mut ledger = ModerationLedger::new();
    let entity = graph.create_entity(EntityKind::Node);
    graph
        .set_property_set(
            entity.id,
            PropertySet::current(Scope::Org, 1, props(&[("name", "acme-org")])),
        )
        .unwrap();

    // Act
    let request_id = ledger
        .submit_change_request(
            &graph,
            entity.id,
            Scope::Org,
            PropertySet::current(Scope::Org, 2, props(&[("name", "acme-org-v2")])),
            "alice",
        )
        .unwrap();

    // Assert: request created as submitted, current truth unchanged
    let request = ledger.get_change_request(request_id).unwrap();
    assert_eq!(request.status, ChangeRequestStatus::Submitted);
    assert_eq!(graph.resolve(entity.id, Scope::Org).unwrap().version, 1);
}

// ------------------------------------------------------------------
// AC-2: Approval + apply makes the proposal current, old version retained
// ------------------------------------------------------------------
#[test]
fn applying_approved_request_makes_proposal_current_and_retains_history() {
    // Arrange
    let mut graph = Graph::new();
    let mut ledger = ModerationLedger::new();
    let entity = graph.create_entity(EntityKind::Node);
    graph
        .set_property_set(
            entity.id,
            PropertySet::current(Scope::Org, 1, props(&[("name", "acme-org")])),
        )
        .unwrap();
    let request_id = ledger
        .submit_change_request(
            &graph,
            entity.id,
            Scope::Org,
            PropertySet::current(Scope::Org, 2, props(&[("name", "acme-org-v2")])),
            "alice",
        )
        .unwrap();
    ledger.decide_request(request_id, true, "reviewer").unwrap();

    // Act
    let outcome = ledger.apply_approved(&mut graph, request_id);

    // Assert: applied, new version current, v1 in history
    assert_eq!(outcome, ApplyOutcome::Applied);
    let current = graph.resolve(entity.id, Scope::Org).unwrap();
    assert_eq!(current.version, 2);
    assert_eq!(current.properties.get("name"), Some(&json!("acme-org-v2")));
    assert_eq!(ledger.history(entity.id, Scope::Org).len(), 1);
    assert_eq!(ledger.history(entity.id, Scope::Org)[0].version, 1);
}

// ------------------------------------------------------------------
// AC-3: Rejection is non-destructive
// ------------------------------------------------------------------
#[test]
fn rejection_changes_nothing() {
    // Arrange
    let mut graph = Graph::new();
    let mut ledger = ModerationLedger::new();
    let entity = graph.create_entity(EntityKind::Node);
    graph
        .set_property_set(
            entity.id,
            PropertySet::current(Scope::Workspace, 1, props(&[("name", "acme-ws")])),
        )
        .unwrap();
    let request_id = ledger
        .submit_change_request(
            &graph,
            entity.id,
            Scope::Workspace,
            PropertySet::current(Scope::Workspace, 2, props(&[("name", "acme-ws-v2")])),
            "alice",
        )
        .unwrap();

    // Act
    ledger
        .decide_request(request_id, false, "reviewer")
        .unwrap();

    // Assert
    assert_eq!(
        graph.resolve(entity.id, Scope::Workspace).unwrap().version,
        1
    );
    assert_eq!(
        ledger.get_change_request(request_id).unwrap().status,
        ChangeRequestStatus::Rejected
    );
}

// ------------------------------------------------------------------
// AC-4: Double-apply is a no-op (idempotent via version guard)
// ------------------------------------------------------------------
#[test]
fn double_apply_is_noop() {
    // Arrange
    let mut graph = Graph::new();
    let mut ledger = ModerationLedger::new();
    let entity = graph.create_entity(EntityKind::Node);
    graph
        .set_property_set(
            entity.id,
            PropertySet::current(Scope::Org, 1, props(&[("name", "acme-org")])),
        )
        .unwrap();
    let request_id = ledger
        .submit_change_request(
            &graph,
            entity.id,
            Scope::Org,
            PropertySet::current(Scope::Org, 2, props(&[("name", "acme-org-v2")])),
            "alice",
        )
        .unwrap();
    ledger.decide_request(request_id, true, "reviewer").unwrap();
    ledger.apply_approved(&mut graph, request_id);

    // Act
    let second = ledger.apply_approved(&mut graph, request_id);

    // Assert
    assert_eq!(second, ApplyOutcome::AlreadyApplied);
    assert_eq!(graph.resolve(entity.id, Scope::Org).unwrap().version, 2);
    assert_eq!(ledger.history(entity.id, Scope::Org).len(), 1);
}

// ------------------------------------------------------------------
// AC-5: Child version promotes to parent with provenance
// ------------------------------------------------------------------
#[test]
fn workspace_version_promotes_to_org_with_provenance() {
    // Arrange
    let mut graph = Graph::new();
    let mut ledger = ModerationLedger::new();
    let entity = graph.create_entity(EntityKind::Node);
    graph
        .set_property_set(
            entity.id,
            PropertySet::current(Scope::Workspace, 3, props(&[("name", "promoted-name")])),
        )
        .unwrap();

    // Act
    let request_id = ledger
        .submit_promotion(&graph, entity.id, Scope::Org, Scope::Workspace, "alice")
        .unwrap();
    ledger.decide_request(request_id, true, "reviewer").unwrap();
    let outcome = ledger.apply_approved(&mut graph, request_id);

    // Assert: org set holds the promoted properties, provenance recorded
    assert_eq!(outcome, ApplyOutcome::Applied);
    let org_set = graph.resolve(entity.id, Scope::Org).unwrap();
    assert_eq!(
        org_set.properties.get("name"),
        Some(&json!("promoted-name"))
    );
    let provenance = ledger.provenance(entity.id, Scope::Org).unwrap();
    assert_eq!(provenance.from_scope, Some(Scope::Workspace));
    assert_eq!(provenance.from_version, Some(3));
    assert_eq!(provenance.request_id, request_id);
}

// ------------------------------------------------------------------
// AC-6: Provenance query returns the full record
// ------------------------------------------------------------------
#[test]
fn provenance_query_returns_full_record() {
    // Arrange + Act
    let mut graph = Graph::new();
    let mut ledger = ModerationLedger::new();
    let entity = graph.create_entity(EntityKind::Node);
    graph
        .set_property_set(
            entity.id,
            PropertySet::current(Scope::Workspace, 3, props(&[("name", "promoted-name")])),
        )
        .unwrap();
    let request_id = ledger
        .submit_promotion(&graph, entity.id, Scope::Org, Scope::Workspace, "alice")
        .unwrap();
    ledger.decide_request(request_id, true, "bob").unwrap();
    ledger.apply_approved(&mut graph, request_id);

    // Assert: full record
    let provenance = ledger.provenance(entity.id, Scope::Org).unwrap();
    assert_eq!(provenance.request_id, request_id);
    assert_eq!(provenance.reviewed_by, "bob");
    assert!(provenance.applied_at.is_some());
    assert_eq!(provenance.from_scope, Some(Scope::Workspace));
    assert_eq!(provenance.from_version, Some(3));
}

// ------------------------------------------------------------------
// Helpers
// ------------------------------------------------------------------
fn props(pairs: &[(&str, &str)]) -> HashMap<String, serde_json::Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), json!(v)))
        .collect()
}
