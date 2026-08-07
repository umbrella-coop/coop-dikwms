# SPEC-012 Feature: Audit & Traceability (platform-wide)

<!-- status: Implemented -->

## Overview

Platform-wide audit machinery: persist the **moderation ledger** (change requests + decisions — currently in-memory, SPEC-002), add **correlation IDs** to trace units of work through the commit stream, **revert markers** so undo operations are auditable, and provide **audit projection queries** (by entity/actor/scope) as Rust functions. The HTTP audit surface is deferred to SPEC-013 (API layer) — same machinery-now/transport-later split as SPEC-004.

**Backlog source:** SPEC-012 (promoted per the Spec Intake Rule; user request).

## Motivation

From the roadmap-fit analysis (docs/brainstorm/graph-conductor-roadmap-fit.md): the RFC's event-store auditability compares to our commit log — which exists (SPEC-006) — but four gaps remain: the moderation ledger is not persisted, no correlation/causation IDs, no revert markers, no audit query surface. Audit is **platform-wide**: manual edits, moderation decisions, promotions, and (later) pipeline batches must all be traceable.

## Requirements

### Requirement: Persisted Moderation Ledger

The system SHALL persist change requests and their decisions as immutable documents, retrievable per entity and per request, with full decision history (who decided, when, approve/reject).

#### Scenario: Request survives restart
- **GIVEN** a change request submitted and decided
- **WHEN** the ledger is reloaded from the store
- **THEN** the request and its decision history are retrievable

### Requirement: Correlation Traceability

The system SHALL accept an optional correlation id on writes, making a unit of work traceable through the commit stream.

#### Scenario: Correlation id is traceable
- **GIVEN** property-set saves sharing a correlation id
- **WHEN** the audit for the entity is queried
- **THEN** all entries carry the correlation id

### Requirement: Auditable Reverts

The system SHALL record a revert as a compensating commit carrying a **revert marker** with the target commit, so undo operations are distinguishable from normal writes.

#### Scenario: Revert is marked
- **GIVEN** a property set reverted to an earlier commit
- **WHEN** the audit is queried
- **THEN** a Reverted entry exists referencing the target commit

### Requirement: Audit Projection Queries

The system SHALL expose audit projections (Rust functions, API later): per entity, per actor, per scope — entries ordered by commit with actor, action, commit, and details.

#### Scenario: Entity audit is ordered and complete
- **GIVEN** a history of creates, saves, decisions, and a revert on an entity
- **WHEN** the entity audit is queried
- **THEN** entries are ordered by commit and include actor/action/commit

## Acceptance Criteria

- AC-1: Given a change request and decision, when persisted and reloaded, then request and decision history are retrievable. ✅
- AC-2: Given decisions, when persisted, then they are append-only (a decision is never mutated) and retrievable. ✅
- AC-3: Given writes with a correlation id, when the entity audit is queried, then entries carry the correlation id. ✅
- AC-4: Given a revert, when audited, then a Reverted entry references the target commit. ✅
- AC-5: Given a history of actions on an entity, when audited, then entries are commit-ordered with actor/action/commit. ✅

**Implementation notes (2026-08-07):** ledger docs registered in `Repository::new`; decision tokens `decision|req:{request_id}:{entity}:{approve}`; revert = compensating save with `|rev:{target}` (version continues upward). Audit projections merge commit-stream events + ledger docs, commit-ordered. `Repository::log` made public.

## Technical Design

### Additions to `terminusdb-repository` (`audit.rs`)

```rust
// persisted ledger documents (immutable, versioned-doc pattern)
pub struct ChangeRequestDoc {
    pub request_id: String, pub entity_id: String, pub instance_id: String,
    pub scope: String, pub proposed_version: u64, pub correlation_id: Option<String>,
    pub created_by: String,
}
pub struct DecisionDoc {
    pub request_id: String, pub decided_by: String, pub approve: bool, pub decided_at: i64,
}

// audit entries (projection over commit log + ledger docs)
pub enum AuditAction { EntityCreated, PropertySetSaved, Reverted, RequestSubmitted, Approved, Rejected }
pub struct AuditEntry { pub commit: String, pub actor: String, pub action: AuditAction,
                        pub entity_id: Uuid, pub correlation_id: Option<String>, pub details: String }

pub async fn save_change_request(...) / save_decision(...)
pub async fn requests_for_entity(repo, entity) -> Vec<(ChangeRequestDoc, Vec<DecisionDoc>)>
pub async fn revert_property_set(repo, entity, instance, target_commit, author, reason) -> Result<()>
pub async fn audit_for_entity(repo, entity) -> Vec<AuditEntry>
pub async fn audit_for_actor(repo, actor) -> Vec<AuditEntry>
```

- **Correlation on writes:** new `save_property_set_correlated(..., correlation_id)` (existing method delegates); token appended to the commit message (`|corr:{id}`) — stream decode ignores unknown tokens
- **Revert:** `resolve_at(entity, chain, target_commit)` → save the restored version with message `{reason}|rev:{target_commit}|ps:...` (version continues upward; history preserved)
- **Ledger docs:** stored as immutable TerminusDB docs; decisions append-only (new DecisionDoc per decision)
- **Actor:** commit `author` (SPEC-003 principal wiring is the API layer's job)

### Out of scope (SPEC-013 / later)
- HTTP audit endpoints, correlation IDs on UI sessions/API calls, batch-level correlation (SPEC-011), principal-verified author enforcement

## Test Plan

- [ ] Integration: ledger round-trip + decision history (AC-1, AC-2)
- [ ] Integration: correlation traceability (AC-3)
- [ ] Integration: revert marker (AC-4)
- [ ] Integration: entity audit ordered + complete (AC-5)
- [ ] Unit: token decode for `|corr:` / `|rev:`; unknown-token tolerance

## Open Risks

| ID | Risk | Impact | Mitigation |
|----|------|--------|------------|
| R-15 | Ledger docs grow with every decision | Low | Immutable docs; per-request queries; index by request id |
| R-16 | Revert chains (revert of a revert) | Medium | `rev:` marker holds the *target* commit; chains remain resolvable via resolve_at |
| R-17 | Audit query cost over long histories | Medium | Filtered projections; snapshot endpoint later (SPEC-013) |

## Relationship to Other Specs

- SPEC-002: ledger persistence closes its in-memory gap; unblocks the activity stream
- SPEC-003: actors become SPEC-003 principals at the API layer
- SPEC-004: commit-stream events + tokens are the audit backbone; corr/rev tokens extend it
- SPEC-006: immutable versioned-doc pattern + resolve_at reused for revert
- SPEC-011: batch correlation rides the same mechanism
- SPEC-013 (planned): HTTP audit surface
