# Generated from: docs/specs/SPEC-003-scope-hierarchy-acl.md
# Generator: /derive bdd v2.0.0
# Generated at: 2026-08-06
# AC Coverage: AC-1..AC-7 (1:1 mapping)

Feature: Scope Hierarchy & Access Control
  [Source] From SPEC-003 Overview
  Scope instances (org/workspace/project) form a parent-child hierarchy;
  membership inherits down; authorization resolves nearest-wins over
  grants; entities attach to multiple scopes for cross-org sharing.

  @SPEC-003 @AC-1
  Scenario: Creating a workspace under an org
    # [Source] From SPEC-003 AC-1
    Given an org scope instance
    When a workspace scope is created with the org as parent
    Then the workspace is recorded as a child of the org

  @SPEC-003 @AC-2
  Scenario: Invalid parent level is rejected
    # [Source] From SPEC-003 AC-2
    Given an org scope instance
    When an org is created with another org as parent
    Then creation fails with a level-order error

  @SPEC-003 @AC-3
  Scenario: Org membership implies workspace membership
    # [Source] From SPEC-003 AC-3
    Given a user is a member of an org scope
    When membership is checked at a descendant workspace scope
    Then the user is a member

  @SPEC-003 @AC-4
  Scenario: Grant at org applies at a descendant project
    # [Source] From SPEC-003 AC-4
    Given an allow-grant for decide at an org scope
    When authorization is checked at a descendant project scope
    Then the action is allowed (inherits down)

  @SPEC-003 @AC-5
  Scenario: Child deny overrides parent allow
    # [Source] From SPEC-003 AC-5
    Given an allow-grant for decide at an org scope
    And a deny-grant for decide at a descendant workspace scope
    When authorization is checked at the workspace scope
    Then the action is denied (nearest-wins)

  @SPEC-003 @AC-6
  Scenario: Entity shared across two organizations
    # [Source] From SPEC-003 AC-6
    Given an entity attached to org-a and org-b
    When attached entities are queried at org-a
    Then the entity is returned
    And querying at org-b also returns the entity
    And org-a content changes do not affect org-b's view

  @SPEC-003 @AC-7
  Scenario: Non-reviewer cannot decide
    # [Source] From SPEC-003 AC-7
    Given a user with no decide_change_request grant at a scope
    When authorize is checked for decide_change_request
    Then the decision is denied
