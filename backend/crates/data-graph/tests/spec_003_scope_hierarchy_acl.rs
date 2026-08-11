// Tests for SPEC-003: Scope Hierarchy & Access Control
// Derived from: docs/specs/SPEC-003-scope-hierarchy-acl.md (Approved 2026-08-06)
// AC Coverage: AC-1..AC-7 (1:1 mapping)

use std::collections::HashMap;

use data_graph::{
    Authorization, COMMON_INSTANCE, EntityKind, Graph, Level, Permission, Policy, PolicyError,
    PropertySet, Scope,
};
use serde_json::json;
use uuid::Uuid;

// ------------------------------------------------------------------
// AC-1: Creating a workspace under an org records the parent-child link
// ------------------------------------------------------------------
#[test]
fn creating_workspace_under_org_records_parent_link() {
    // Arrange
    let mut policy = Policy::new();
    let org = policy.create_scope(Level::Org, "acme", None).unwrap();

    // Act
    let ws = policy
        .create_scope(Level::Workspace, "marketing", Some(org))
        .unwrap();

    // Assert
    assert_eq!(policy.scope(ws).unwrap().parent, Some(org));
    assert_eq!(policy.scope(ws).unwrap().level, Level::Workspace);
}

// ------------------------------------------------------------------
// AC-2: Invalid parent level is rejected
// ------------------------------------------------------------------
#[test]
fn org_with_org_parent_is_rejected() {
    // Arrange
    let mut policy = Policy::new();
    let org_a = policy.create_scope(Level::Org, "acme-a", None).unwrap();

    // Act
    let result = policy.create_scope(Level::Org, "acme-b", Some(org_a));

    // Assert
    assert_eq!(result, Err(PolicyError::InvalidParentLevel));
}

// ------------------------------------------------------------------
// AC-3: Org membership implies descendant workspace membership
// ------------------------------------------------------------------
#[test]
fn org_membership_implies_workspace_membership() {
    // Arrange
    let mut policy = Policy::new();
    let org = policy.create_scope(Level::Org, "acme", None).unwrap();
    let ws = policy
        .create_scope(Level::Workspace, "marketing", Some(org))
        .unwrap();
    let user = Uuid::now_v7();

    // Act
    policy.add_membership(user, org);
    let is_member = policy.is_member(user, ws);

    // Assert
    assert!(is_member);
}

// ------------------------------------------------------------------
// AC-4: Grant at org applies at a descendant project (inherits down)
// ------------------------------------------------------------------
#[test]
fn grant_at_org_applies_at_descendant_project() {
    // Arrange
    let mut policy = Policy::new();
    let org = policy.create_scope(Level::Org, "acme", None).unwrap();
    let ws = policy
        .create_scope(Level::Workspace, "marketing", Some(org))
        .unwrap();
    let project = policy
        .create_scope(Level::Project, "campaign", Some(ws))
        .unwrap();
    let user = Uuid::now_v7();
    policy
        .set_grant(org, user, Permission::DecideChangeRequest, true)
        .unwrap();

    // Act
    let decision = policy.authorize(user, project, Permission::DecideChangeRequest);

    // Assert
    assert_eq!(decision, Authorization::Allow);
}

// ------------------------------------------------------------------
// AC-5: Child deny overrides parent allow (nearest-wins)
// ------------------------------------------------------------------
#[test]
fn child_deny_overrides_parent_allow() {
    // Arrange
    let mut policy = Policy::new();
    let org = policy.create_scope(Level::Org, "acme", None).unwrap();
    let ws = policy
        .create_scope(Level::Workspace, "marketing", Some(org))
        .unwrap();
    let user = Uuid::now_v7();
    policy
        .set_grant(org, user, Permission::DecideChangeRequest, true)
        .unwrap();
    policy
        .set_grant(ws, user, Permission::DecideChangeRequest, false)
        .unwrap();

    // Act
    let decision = policy.authorize(user, ws, Permission::DecideChangeRequest);

    // Assert
    assert_eq!(decision, Authorization::Deny);
}

// ------------------------------------------------------------------
// AC-6: Entity attached to two orgs is visible in both; content isolated
// ------------------------------------------------------------------
#[test]
fn entity_shared_across_orgs_with_isolated_content() {
    // Arrange
    let mut policy = Policy::new();
    let mut graph = Graph::new();
    let org_a = policy.create_scope(Level::Org, "acme-a", None).unwrap();
    let org_b = policy.create_scope(Level::Org, "acme-b", None).unwrap();
    let entity = graph.create_entity(EntityKind::Node);

    // Act: attach to both orgs; org-a writes its own property set
    policy.attach_entity(entity.id, org_a);
    policy.attach_entity(entity.id, org_b);
    graph
        .set_property_set(
            entity.id,
            org_a,
            PropertySet::current(Scope::Org, 1, props(&[("name", "org-a-only")])),
        )
        .unwrap();

    // Assert: visible at both orgs (shared)
    assert_eq!(policy.entities_in_scope(org_a), vec![entity.id]);
    assert_eq!(policy.entities_in_scope(org_b), vec![entity.id]);

    // Assert: content isolated — org-b sees nothing, org-a sees its own set
    let chain_a = vec![org_a, COMMON_INSTANCE];
    let chain_b = vec![org_b, COMMON_INSTANCE];
    assert_eq!(
        graph.resolve(entity.id, &chain_a).unwrap().properties["name"],
        json!("org-a-only")
    );
    assert!(graph.resolve(entity.id, &chain_b).is_none());
}

// ------------------------------------------------------------------
// AC-7: Unprivileged user is denied
// ------------------------------------------------------------------
#[test]
fn unprivileged_user_is_denied() {
    // Arrange
    let mut policy = Policy::new();
    let org = policy.create_scope(Level::Org, "acme", None).unwrap();
    let user = Uuid::now_v7();

    // Act
    let decision = policy.authorize(user, org, Permission::DecideChangeRequest);

    // Assert
    assert_eq!(decision, Authorization::Deny);
}

// ------------------------------------------------------------------
// R-6 guard: no cycles (leaf-only creation; parent must exist)
// ------------------------------------------------------------------
#[test]
fn missing_parent_scope_is_rejected() {
    // Arrange
    let mut policy = Policy::new();

    // Act: parent id that does not exist
    let result = policy.create_scope(Level::Workspace, "ghost", Some(Uuid::now_v7()));

    // Assert
    assert_eq!(result, Err(PolicyError::ScopeNotFound));
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
