# Generated from: docs/specs/SPEC-001-core-graph-model.md
# Generator: /derive bdd v2.0.0
# Generated at: 2026-08-06
# AC Coverage: AC-1..AC-6 (1:1 mapping)

Feature: Core Graph Model — Identity & Scoped Knowledge
  [Source] From SPEC-001 Overview
  The platform assigns every node, edge and combo a global immutable identity,
  and stores its content in scoped property sets (common/org/workspace/project)
  resolved through a pure resolve function.

  @SPEC-001 @AC-1
  Scenario: Entity creation yields a stable global identity without content
    # [Source] From SPEC-001 AC-1
    Given an organization creates a node
    When the node is persisted
    Then the node has a global UUIDv7 identifier that never changes
    And the node has no property set at any scope

  @SPEC-001 @AC-2
  Scenario: Resolution falls back to a parent scope when the queried scope has no property set
    # [Source] From SPEC-001 AC-2
    Given an entity with a property set at common scope
    And an entity with a property set at org scope
    And no property set at workspace scope
    When the entity is resolved at workspace scope
    Then the org-level property set is returned

  @SPEC-001 @AC-3
  Scenario: Resolution overrides the parent with the nearest scope property set
    # [Source] From SPEC-001 AC-3
    Given an entity with a property set at org scope
    And an entity with a property set at workspace scope
    When the entity is resolved at workspace scope
    Then the workspace-level property set is returned

  @SPEC-001 @AC-4
  Scenario: Soft-deleted property set is ignored by resolution
    # [Source] From SPEC-001 AC-4
    Given an entity with a property set at org scope
    And a soft-deleted property set at workspace scope
    When the entity is resolved at workspace scope
    Then the org-level property set is returned

  @SPEC-001 @AC-5
  Scenario: Schema change regenerates storage schema and DTO artifacts
    # [Source] From SPEC-001 AC-5
    Given a source schema based on schema.org types
    When the source schema is modified and the build runs
    Then the generated TerminusDB schema and Rust DTOs are regenerated
    And the generated artifacts compile

  @SPEC-001 @AC-6
  Scenario: Node storage round-trips all scope layers without data loss
    # [Source] From SPEC-001 AC-6
    Given the generated storage schema is applied
    When a node with property sets at all four scopes is stored and retrieved
    Then all scope layers round-trip without data loss
