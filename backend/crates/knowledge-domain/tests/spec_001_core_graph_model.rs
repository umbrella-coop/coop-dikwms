// Tests for SPEC-001: Core Graph Model — Identity & Scoped Knowledge
// Derived from: docs/specs/SPEC-001-core-graph-model.md (Approved 2026-08-06)
// TDD: RED phase — these tests target the public API of `knowledge-domain`.
// AC Coverage: AC-1..AC-6 (1:1 mapping)

use std::collections::HashMap;

use knowledge_domain::{EntityKind, Graph, PropertySet, Scope};
use serde_json::json;

// ------------------------------------------------------------------
// AC-1: Entity creation yields a stable global identity without content
// ------------------------------------------------------------------
#[test]
fn entity_creation_yields_stable_uuidv7_without_property_set() {
    // Arrange / Act
    let mut graph = Graph::new();
    let entity = graph.create_entity(EntityKind::Node);

    // Assert: UUIDv7 (version nibble = 7)
    assert_eq!(entity.id.get_version(), Some(uuid::Version::SortRand));

    // Assert: no property set at any scope
    for scope in [Scope::Common, Scope::Org, Scope::Workspace, Scope::Project] {
        assert!(graph.resolve(entity.id, scope).is_none());
    }

    // Assert: identity is stable on re-read
    assert_eq!(graph.get_entity(entity.id), Some(&entity));
}

// ------------------------------------------------------------------
// AC-2: Resolution falls back to a parent scope when queried scope has no set
// ------------------------------------------------------------------
#[test]
fn resolve_returns_org_set_when_workspace_has_no_set() {
    // Arrange
    let mut graph = Graph::new();
    let entity = graph.create_entity(EntityKind::Node);

    graph
        .set_property_set(
            entity.id,
            PropertySet::current(Scope::Common, 1, props(&[("name", "acme")])),
        )
        .unwrap();
    graph
        .set_property_set(
            entity.id,
            PropertySet::current(Scope::Org, 1, props(&[("name", "acme-org")])),
        )
        .unwrap();

    // Act
    let resolved = graph.resolve(entity.id, Scope::Workspace).unwrap();

    // Assert: nearest scope above the query that has a current set
    assert_eq!(resolved.scope, Scope::Org);
    assert_eq!(resolved.properties.get("name"), Some(&json!("acme-org")));
}

// ------------------------------------------------------------------
// AC-3: Resolution overrides the parent with the nearest scope set
// ------------------------------------------------------------------
#[test]
fn resolve_returns_workspace_set_when_present_over_org() {
    // Arrange
    let mut graph = Graph::new();
    let entity = graph.create_entity(EntityKind::Node);

    graph
        .set_property_set(
            entity.id,
            PropertySet::current(Scope::Org, 1, props(&[("name", "acme-org")])),
        )
        .unwrap();
    graph
        .set_property_set(
            entity.id,
            PropertySet::current(Scope::Workspace, 1, props(&[("name", "acme-ws")])),
        )
        .unwrap();

    // Act
    let resolved = graph.resolve(entity.id, Scope::Workspace).unwrap();

    // Assert: nearest-wins
    assert_eq!(resolved.scope, Scope::Workspace);
    assert_eq!(resolved.properties.get("name"), Some(&json!("acme-ws")));
}

// ------------------------------------------------------------------
// AC-4: Soft-deleted property set is ignored by resolution
// ------------------------------------------------------------------
#[test]
fn resolve_skips_soft_deleted_workspace_set_and_returns_org() {
    // Arrange
    let mut graph = Graph::new();
    let entity = graph.create_entity(EntityKind::Node);

    graph
        .set_property_set(
            entity.id,
            PropertySet::current(Scope::Org, 1, props(&[("name", "acme-org")])),
        )
        .unwrap();
    graph
        .set_property_set(
            entity.id,
            PropertySet::current(Scope::Workspace, 1, props(&[("name", "acme-ws")])),
        )
        .unwrap();
    graph.soft_delete(entity.id, Scope::Workspace);

    // Act
    let resolved = graph.resolve(entity.id, Scope::Workspace).unwrap();

    // Assert: soft-deleted set skipped, parent returned
    assert_eq!(resolved.scope, Scope::Org);
    assert_eq!(resolved.properties.get("name"), Some(&json!("acme-org")));
}

// ------------------------------------------------------------------
// AC-5: Schema change regenerates storage schema and DTO artifacts
// ------------------------------------------------------------------
// [TODO] Gated on schema pipeline (Open Risk R-2) — schema.org source + codegen toolchain.
#[test]
#[ignore = "R-2: schema pipeline not chosen yet; R-1: TerminusDB unverified"]
fn generated_dtos_compile_and_reflect_source_schema() {
    // [TODO] Build with modified source schema; assert regeneration compiles
}

// ------------------------------------------------------------------
// AC-6: Node storage round-trips all scope layers without data loss
// ------------------------------------------------------------------
// [TODO] Gated on persistence layer (Open Risk R-1) — TerminusDB single store unverified.
#[test]
#[ignore = "R-1: TerminusDB single store unverified (terminusdb.com HTTP 522)"]
fn stored_node_round_trips_four_scope_layers() {
    // [TODO] Persist via repository; assert all four scope layers round-trip
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
