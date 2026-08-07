# SPEC-003 Feature: Scope Hierarchy & Access Control

<!-- status: Archived -->

## Overview

Introduce **scope instances** (organizations, workspaces, projects) as first-class nodes with a parent-child hierarchy (`org → workspace → project`), **membership** inheritance, and a **policy-based access control** model (grants per principal × scope × permission, with parent inheritance and nearest-wins deny). Entities (SPEC-001) can be **attached to multiple scope instances** — the mechanism by which nodes/edges/combos are *shared, searched, and used across organizations, workspaces, and projects*.

**Vocabulary note:** SPEC-001's `Scope` is a ladder *level* (`common/org/workspace/project`); SPEC-003's scope is an *instance* (e.g. `acme-org`, `acme-org/marketing-ws`). Both coexist — a scope instance always lives at a ladder level.

## Motivation

From the brainstorm (docs/brainstorm/knowledge-graph-platform.md, idea #6): *"org/workspace/project as scope nodes with `memberOf` edges + ACL attached to scope nodes; 'shared across orgs' = a query, not a join."* SPEC-002 introduced moderation (R-4: reviewer is an untyped `String`); SPEC-003 supplies the principals, roles, and permission checks that gate `decide_request` and other actions.

## Requirements

### Requirement: Scope Hierarchy

The system SHALL model scope instances (org/workspace/project) as nodes with parent links, enforcing level ordering: org has no parent, workspace's parent is an org, project's parent is a workspace.

#### Scenario: Creating a workspace under an org
- **GIVEN** an org scope instance
- **WHEN** a workspace scope is created with the org as parent
- **THEN** the workspace is recorded as a child of the org

#### Scenario: Invalid parent level is rejected
- **GIVEN** an org scope instance
- **WHEN** an org is created with another org as parent
- **THEN** creation fails with a level-order error

### Requirement: Membership Inheritance

The system SHALL treat a user as a member of every descendant scope of any scope they belong to.

#### Scenario: Org membership implies workspace membership
- **GIVEN** a user is a member of an org scope
- **WHEN** membership is checked at a descendant workspace scope
- **THEN** the user is a member

### Requirement: Permission Inheritance (nearest-wins)

The system SHALL resolve authorization by walking the scope and its ancestors: the **nearest** explicit decision (allow or deny) wins; no match yields `Deny`.

#### Scenario: Grant at org applies at a descendant project
- **GIVEN** an allow-grant for `decide` at an org scope
- **WHEN** authorization is checked at a descendant project scope
- **THEN** the action is allowed (inherits down)

#### Scenario: Child deny overrides parent allow
- **GIVEN** an allow-grant for `decide` at an org scope
- **AND** a deny-grant for `decide` at a descendant workspace scope
- **WHEN** authorization is checked at the workspace scope
- **THEN** the action is denied (nearest-wins)

### Requirement: Entity Attachment & Cross-Org Sharing

The system SHALL allow an entity to be attached to multiple scope instances, and SHALL expose which entities are attached to a scope.

#### Scenario: Entity shared across two organizations
- **GIVEN** an entity attached to org-a and org-b
- **WHEN** attached entities are queried at org-a
- **THEN** the entity is returned
- **AND** querying at org-b also returns the entity

#### Scenario: Content isolation per attachment
- **GIVEN** an entity attached to org-a and org-b
- **WHEN** org-a applies a change request to the entity's org-a scope
- **THEN** org-b's resolved view of the entity is unaffected

### Requirement: Authorization for Moderation (SPEC-002 integration)

The system SHALL expose an `authorize(principal, scope_id, permission)` function that gates SPEC-002's `decide_request` (permission: `decide_change_request`) and submission (`submit_change_request`).

#### Scenario: Non-reviewer cannot decide
- **GIVEN** a user with no `decide_change_request` grant at a scope
- **WHEN** `authorize` is checked for `decide_change_request`
- **THEN** the decision is `Deny`

## Acceptance Criteria

- AC-1: Given an org scope, when a workspace is created under it, then the parent-child link is recorded.
- AC-2: Given an org scope, when an org is created with an org parent, then creation fails (level-order error).
- AC-3: Given a user member of an org scope, when membership is checked at a descendant workspace, then the user is a member.
- AC-4: Given an allow-grant for `decide` at an org scope, when checked at a descendant project scope, then the action is allowed.
- AC-5: Given an allow-grant at org and a deny-grant at a descendant workspace, when checked at the workspace, then the action is denied (nearest-wins).
- AC-6: Given an entity attached to org-a and org-b, when attached entities are queried at org-a, then the entity is returned; content changes at org-a do not affect org-b's view.
- AC-7: Given a user with no `decide_change_request` grant at a scope, when `authorize` is checked, then the decision is `Deny`.

## Technical Design

### Domain additions (knowledge-domain crate)

```rust
pub enum Level { Org, Workspace, Project }

pub struct ScopeNode {
    pub id: Uuid,              // UUIDv7
    pub level: Level,
    pub name: String,
    pub parent: Option<Uuid>,  // None only for org roots
}

pub enum Permission { SubmitChangeRequest, DecideChangeRequest, Read, Write, ManageScope }

pub enum Authorization { Allow, Deny }

pub struct Grant {
    pub scope_id: Uuid,
    pub principal: Uuid,       // user id (replaces String reviewer — closes R-4)
    pub permission: Permission,
    pub allow: bool,
}

pub struct Policy {
    scopes: HashMap<Uuid, ScopeNode>,
    memberships: HashMap<Uuid, Vec<Uuid>>,   // user -> scope ids
    grants: HashMap<(Uuid, Uuid, Permission), bool>, // (scope, principal, permission) -> allow
    attachments: HashMap<Uuid, Vec<Uuid>>,   // entity -> scope ids
}
```

```rust
pub fn create_scope(&mut self, level: Level, name: &str, parent: Option<Uuid>)
    -> Result<Uuid, PolicyError>;           // validates level ordering
pub fn add_membership(&mut self, principal: Uuid, scope_id: Uuid);
pub fn is_member(&self, principal: Uuid, scope_id: Uuid) -> bool;   // ancestor walk
pub fn set_grant(&mut self, scope_id: Uuid, principal: Uuid, permission: Permission, allow: bool);
pub fn authorize(&self, principal: Uuid, scope_id: Uuid, permission: Permission) -> Authorization; // nearest-wins
pub fn attach_entity(&mut self, entity_id: Uuid, scope_id: Uuid);
pub fn entities_in_scope(&self, scope_id: Uuid) -> Vec<Uuid>;
```

### Resolution rules
- `authorize`: walk `scope_id` upward (self, parent, ...); the first grant found for (principal, permission) wins (allow or deny); no grant → `Deny`.
- `is_member`: own membership or membership of any ancestor scope.
- Level-order: `Org` parent must be `None`; `Workspace` parent must be `Org`; `Project` parent must be `Workspace` (`PolicyError::InvalidParentLevel`).
- Content isolation across attachments (AC-6) is guaranteed by SPEC-001's per-level ladder: org-a and org-b are distinct ladder scopes at the same level, so their property sets are independent.

### Integration (application layer, future spec)
- SPEC-002's `decide_request`/`submit_change_request` call `Policy::authorize` before acting (application layer wiring; the ledger stays pure). Reviewer type migrates from `String` to principal `Uuid` (closes R-4).

## Test Plan

- [ ] Unit: level-order validation (org/workspace/project parents) — AC-1, AC-2
- [ ] Unit: membership inheritance via ancestor walk — AC-3
- [ ] Unit: authorization nearest-wins (allow down, deny closer) — AC-4, AC-5
- [ ] Unit: cross-org attachment + content isolation — AC-6
- [ ] Unit: unprivileged authorization denied — AC-7
- [ ] Property test: authorize is deterministic; member set is transitive

## Open Risks

| ID | Risk | Impact | Mitigation |
|----|------|--------|------------|
| R-5 | Roles (admin/editor/reviewer/viewer) are implicit grants until a role system is defined | Low | `set_grant` composes; role presets deferred to application layer |
| R-6 | Cycles in scope hierarchy not checked (parent chain walks may loop) | Medium | Parent validation + cycle guard in `create_scope` (implementation must verify) |

## Relationship to Other Specs

- SPEC-001: scope instances live at ladder levels; content isolation via ladder
- SPEC-002: `authorize` gates moderation; closes R-4 (principal replaces String reviewer)
- SPEC-004 (planned): live streaming — scope-scoped event subscriptions use `entities_in_scope`


---

## ARCHIVED (2026-08-07)

- **Verification commit:** `d01ccb4`
- **Evidence:** backend/crates/knowledge-domain/tests/spec_003_scope_hierarchy_acl.rs
- **ACs:** 8/8 verified green against real TerminusDB 12.1 (TerminusDBServer pattern)
- **Status change:** Implemented → Archived. Re-check (spec-vs-code convergence) if touched by future work.


---

## ARCHIVED (2026-08-07)

- **Verification commit:** `d01ccb4`
- **Evidence:** backend/crates/knowledge-domain/tests/spec_003_scope_hierarchy_acl.rs
- **ACs:** 8/8 verified green against real TerminusDB 12.1 (TerminusDBServer pattern)
- **Status change:** → Archived. Re-check (spec-vs-code convergence) if touched by future work.
