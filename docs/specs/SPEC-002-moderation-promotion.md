# SPEC-002 Feature: Moderation & Promotion Workflow

<!-- status: Approved -->

## Overview

Add the request-to-change moderation process and the promotion ladder to the knowledge graph platform. Any change to a scope's property set (SPEC-001) must flow through a **change request** (submitted → approved/rejected), and a **child scope's** version may be **promoted to its parent scope** through the same process. Applied promotions record **provenance** (source scope, source version, request, reviewer). This spec is pure domain logic in the `knowledge-domain` crate — no storage dependency (TerminusDB verification remains the SPEC-001 gate).

## Motivation

From the brainstorm (docs/brainstorm/knowledge-graph-platform.md): *shared truth must be stable, local truth must be adaptable*. SPEC-001 gives identity + scoped property sets; SPEC-002 gives the **governance path** — how a layer of truth decides to *use a parent source* or *use a promoted version from a child layer*, safely, auditably, and idempotently (brainstorm ideas: #3 Git-for-Knowledge-Layers, #6 Idempotent Moderation Apply, #10 Provenance-as-Edges).

## Requirements

### Requirement: Change Request Submission

The system SHALL record any proposal to change a scope's property set as a **change request** with status `submitted`, carrying the proposed property set. The proposal SHALL NOT alter the current property set.

#### Scenario: Submitting a change request does not change current truth
- **GIVEN** an entity with a current org-level property set version 1
- **WHEN** a change request proposing org version 2 is submitted
- **THEN** a change request with status `submitted` is created
- **AND** the org-level set remains version 1

### Requirement: Approval Applies the Change

The system SHALL, upon approval of a change request, apply the proposed property set atomically as the new current version. The superseded version SHALL be retained as history.

#### Scenario: Approved request becomes current truth
- **GIVEN** an approved change request proposing org version 2
- **WHEN** the request is applied
- **THEN** the org-level property set becomes version 2 with status `current`
- **AND** version 1 remains retrievable as history

### Requirement: Rejection Is Non-Destructive

The system SHALL NOT alter any property set when a change request is rejected.

#### Scenario: Rejected request leaves truth untouched
- **GIVEN** a rejected change request proposing workspace version 3
- **WHEN** the rejection is finalized
- **THEN** the workspace-level property set is unchanged
- **AND** the request status is `rejected`

### Requirement: Idempotent Apply

The system SHALL make applying an approved change request idempotent via a version guard: a request whose proposed version is not newer than the current version SHALL NOT be applied twice.

#### Scenario: Double-apply is a no-op
- **GIVEN** an already-applied change request
- **WHEN** apply is invoked again
- **THEN** no property set changes
- **AND** the outcome reports the request was already applied

### Requirement: Promotion from a Child Scope

The system SHALL allow a child scope's current version to be proposed as a promotion to its parent scope. The promoted set SHALL copy the child's properties and SHALL carry provenance.

#### Scenario: Workspace version promotes to org
- **GIVEN** a workspace-level property set version 3
- **WHEN** a change request promoting workspace version 3 to org scope is approved and applied
- **THEN** the org-level property set contains the workspace version 3 properties
- **AND** its provenance records `from workspace, version 3`

### Requirement: Provenance Query

The system SHALL record and expose provenance for every applied change: request id, reviewed by, applied at, and (for promotions) source scope and source version.

#### Scenario: Provenance of a promoted set is retrievable
- **GIVEN** a promoted org-level property set
- **WHEN** its provenance is queried
- **THEN** source scope, source version, request id, reviewer, and applied-at time are returned

## Acceptance Criteria

- AC-1: Given a current org set v1, when a request proposing org v2 is submitted, then the request is `submitted` and the org set stays v1.
- AC-2: Given an approved request proposing org v2, when applied, then the org set becomes current v2 and v1 is retained as history.
- AC-3: Given a rejected request, when finalized, then no property set changes and status is `rejected`.
- AC-4: Given an already-applied request, when apply is invoked again, then nothing changes and the outcome is `already-applied`.
- AC-5: Given a workspace set v3, when a promotion request to org is approved and applied, then the org set holds v3's properties with provenance `from workspace v3`.
- AC-6: Given a promoted org set, when provenance is queried, then source scope, source version, request id, reviewer, and applied-at are returned.

## Technical Design

### Domain additions (knowledge-domain crate)

```rust
pub enum ChangeRequestStatus { Submitted, UnderReview, Approved, Rejected, Applied }

pub struct ChangeRequest {
    pub id: Uuid,                    // UUIDv7
    pub entity_id: Uuid,
    pub scope: Scope,
    pub proposed: PropertySet,       // candidate set to be applied on approval
    pub status: ChangeRequestStatus,
    pub created_by: String,
    pub reviewed_by: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub applied_at: Option<DateTime<Utc>>,
}

pub enum ApplyOutcome { Applied, AlreadyApplied, NotApproved }

pub struct Provenance {
    pub request_id: Uuid,
    pub reviewed_by: String,
    pub applied_at: DateTime<Utc>,
    pub from_scope: Option<Scope>,   // Some(..) for promotions
    pub from_version: Option<u64>,
}
```

`Graph` gains:

```rust
pub fn submit_change_request(&mut self, entity_id, scope, proposed: PropertySet, created_by: &str)
    -> Result<Uuid, SetError>            // returns request id; validates entity + version ordering
pub fn decide_request(&mut self, request_id, approve: bool, reviewed_by: &str) -> Result<(), ...>
pub fn apply_approved(&mut self, request_id) -> ApplyOutcome     // version-guarded, idempotent
pub fn provenance(&self, entity_id, scope) -> Option<&Provenance>
```

### Version semantics
- Reuses SPEC-001's monotonic version + `Status::Current/Retired`; rejected/candidate sets never become `Current`.
- Apply writes the proposed set (bumping storage), keeps the previous version as history (SPEC-001 keeps only latest per scope — **history retention is a delta**: keep a `history: Vec<PropertySet>` per (entity, scope) in v1 in-memory; persistence arrives with R-1).

### Conflict detection (deferred)
Two children promoting incompatible versions of the same property to the parent is handled by the version guard (AC-4) in v1; richer diff/merge conflict UX is out of scope (brainstorm: scoped to property-diff in a later spec).

## Test Plan

- [ ] Unit: submission creates request, does not alter current set (AC-1)
- [ ] Unit: apply of approved request makes proposed version current, old retained (AC-2)
- [ ] Unit: rejection changes nothing (AC-3)
- [ ] Unit: double-apply idempotent via version guard (AC-4)
- [ ] Unit: promotion copies child properties + provenance (AC-5)
- [ ] Unit: provenance query returns full record (AC-6)
- [ ] Property test: apply is idempotent for any request (run twice → same state)

## Open Risks

| ID | Risk | Impact | Mitigation |
|----|------|--------|------------|
| R-3 | History retention (per-scope version history) grows unbounded | Low in v1 (in-memory) | Persistence + retention policy with R-1 (SPEC-001) |
| R-4 | Reviewer/actor identity is a plain String until auth (Postgres) is wired | Low | Replace with user-id type when auth lands (SPEC-003) |

## Relationship to Other Specs

- SPEC-001: consumes `PropertySet` version/status semantics; adds history
- SPEC-003 (planned): scope hierarchy + ACL — reviewer authorization for `decide_request`
- SPEC-004 (planned): live streaming — change-request events (submitted/approved/applied) become domain events

---

## MODIFIED Requirements (delta — 2026-08-07)

- **MODIFIED — History retention (R-3):** per-scope `history: Vec<PropertySet>` is superseded by the **TerminusDB commit graph** (SPEC-006) — diffs/time-travel replace the in-memory vector; the `history()` API remains a convenience projection over immutable versioned documents.
- **MODIFIED — Ledger persistence (deferred):** the `ModerationLedger` remains in-memory in v1; persisting change requests + decisions is **SPEC-012 (Audit & Traceability)** and also unblocks the activity stream.
