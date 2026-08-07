# SPEC-001 Feature: Core Graph Model — Identity & Scoped Knowledge

<!-- status: Approved -->

## Overview

Define the core graph data model of the platform: graph primitives (node, edge, combo) with **global, immutable identity** and **scope-agnostic content resolution**. Every entity is unique across the whole organization (and beyond), while its content lives in **scoped property sets** — `common`, `org`, `workspace`, `project` — resolved through a pure `resolve(scope)` function. This spec covers the data model and resolution semantics only; the moderation/promotion workflow (request-to-change, promotion ladder, approval edges) is SPEC-002.

## Motivation

From the brainstorm (docs/brainstorm/knowledge-graph-platform.md): *shared truth must be stable, local truth must be adaptable*. Without identity/content separation, sharing a node across organizations forces either duplicate entities (unmanageable governance) or shared mutable state (uncontrolled change). The scoped-property-set model gives one identity, four layers of controlled truth, and a pure resolution function that later specs (streaming, moderation, visualization) can rely on.

## Requirements

### Requirement: Global Identity

The system SHALL assign every node, edge, and combo a single immutable global identifier (UUIDv7), unique across all organizations.

#### Scenario: Entity creation yields unique identity
- **GIVEN** an organization creates a node
- **WHEN** the node is persisted
- **THEN** it has a global UUID that never changes for its lifetime

### Requirement: Scoped Property Sets

The system SHALL store entity content as scoped property sets, one per scope level: `common`, `org`, `workspace`, `project`. Each property set SHALL carry a monotonic version number and a `status` (candidate/current/retired).

#### Scenario: A scope has its own property set
- **GIVEN** an entity with an org-level property set
- **WHEN** a workspace member queries the entity at workspace scope
- **THEN** resolution falls back to the org-level set (or higher) if no workspace-level set exists

#### Scenario: Soft delete is per-scope
- **GIVEN** an entity visible at both org and workspace scope
- **WHEN** the workspace scope is soft-deleted
- **THEN** the entity remains resolvable at org scope

### Requirement: Content Resolution (resolve function)

The system SHALL resolve the effective property set for a given scope via a pure function `resolve(entity, scope) → propertySet` that walks `project → workspace → org → common`, returning the nearest scope with a current property set, or `null`.

#### Scenario: Resolution falls back to parent
- **GIVEN** an entity with property sets at `common` and `org` only
- **WHEN** `resolve(entity, workspace)` is called
- **THEN** the org-level set is returned

#### Scenario: Resolution overrides parent
- **GIVEN** an entity with property sets at `org` and `workspace`
- **WHEN** `resolve(entity, workspace)` is called
- **THEN** the workspace-level set is returned (nearest-wins)

### Requirement: Schema-Driven Model

The system SHALL model entities from a **single source schema** based on schema.org types (e.g. `Person`, `Organization`, `Project` extended with graph primitives), from which the storage schema (TerminusDB) and Rust DTOs are **generated as build artifacts**.

#### Scenario: Schema change regenerates artifacts
- **GIVEN** the source schema is modified
- **WHEN** the build runs
- **THEN** generated storage schema and Rust DTOs are regenerated and compile

## Acceptance Criteria

- AC-1: Given entity creation, when persisted, then the entity has a stable global UUIDv7 and no property set yet.
- AC-2: Given entity with sets at `common` and `org`, when `resolve(entity, workspace)` runs, then the org set is returned.
- AC-3: Given entity with sets at `org` and `workspace`, when `resolve(entity, workspace)` runs, then the workspace set is returned.
- AC-4: Given a soft-deleted workspace set, when resolving at workspace scope, then the parent scope's set is returned.
- AC-5: Given a changed source schema, when the build runs, then generated artifacts compile.
- AC-6: Given the generated storage schema, when a node is stored, then all scope layers round-trip without data loss.

## Technical Design

### Stores (v1)
- **TerminusDB** (single system of record): graph data + scoped property sets + versions.
- **PostgreSQL**: authentication/sessions only (unchanged by this spec).
- Event-sourcing (SPEC-004) is the deferred v2 target — the `resolve` function is designed pure so it can later consume an event log.

### Identity
- UUIDv7 for global identifiers (time-ordered, index-friendly, no coordination needed).

### Scoped property sets
- One document per (entity, scope) holding: `scope`, `version` (monotonic), `status`, `properties` (schema.org-typed fields), `created_at`, `updated_at`, `soft_deleted_at`.
- `resolve(entity, scope)` implemented as a pure traversal over scope-level documents with an in-memory cache keyed by `(entity_id, scope, version)`.

### Schema pipeline
- Source: `schemas/schema.org.graphql` or JSON-LD subset — **decision deferred to implementation** (see Open Risks R-2).
- Generated: TerminusDB `@to` schema + Rust DTOs via a codegen step in the build.

### Rust backend structure (initial)
- Hexagonal: `domain` (entity, scope, resolve), `application` (commands/queries), `infrastructure` (TerminusDB repository, Postgres auth).

## Test Plan

- [ ] Unit: `resolve` fallback and override matrix (all 4 scope levels × presence/absence)
- [ ] Unit: version monotonicity; soft-delete isolation per scope
- [ ] Integration: entity CRUD round-trip through TerminusDB incl. all 4 scope layers
- [ ] Integration: schema regeneration → build passes (codegen pipeline)
- [ ] Property test: `resolve` is deterministic and pure for identical inputs

## Open Risks

| ID | Risk | Impact | Mitigation |
|----|------|--------|------------|
| R-1 | TerminusDB capabilities — **partially verified** (github.com/terminusdb/terminusdb README, Aug 2026): commits/diff/push-pull ✓, time-travel queries ✓, Allen-interval temporal reasoning (v12) ✓, JSON Git-for-Data ✓, WOQL/GraphQL/REST ✓, Rust client (ParaplouOU/terminusdb-rs) ✓; live subscriptions ✗ (unconfirmed). Site terminusdb.com still HTTP 522 | High — gates persistence layer | Verify remaining unknowns (@to schema specifics, Rust client maturity, subscription availability) hands-on via local Docker server; fallback: PostgreSQL-first with graph projection later |
| R-2 | Codegen toolchain (schema.org → @to → Rust DTOs) not chosen | Medium | Spike 2 candidate toolchains during IMPLEMENTATION |
| R-3 | schema.org coverage for edge/combo semantics incomplete | Medium | Define custom extension types in source schema |

## Relationship to Other Specs

- SPEC-002 (planned): Moderation & Promotion workflow (request-to-change, promotion ladder, provenance) — consumes `status`/`version`
- SPEC-003 (planned): Scope hierarchy (org/workspace/project scope nodes + ACL)
- SPEC-004 (planned): Live streaming (CDC + cursor) — consumes `version` for cursors

## References

- Brainstorm report: docs/brainstorm/knowledge-graph-platform.md (ideas #1 Identity/Content Separation, #12 Schema-First Codegen)
- Bit docs (grounded, Aug 2026): https://bit.dev/docs/intro
