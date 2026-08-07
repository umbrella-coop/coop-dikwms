# SPEC-013 Feature: API Layer (axum REST)

<!-- status: Archived -->

## Overview

The platform's HTTP surface: an axum application composing the eight implemented specs — core CRUD (entities, scoped property sets, resolve), moderation (submit/decide/apply/promote), audit queries, schema-registry namespace listing + `additionalType` warning, and the SPEC-004 SSE stream. v1 auth = a simple principal header; verified principals (SPEC-003) and sessions are documented risks. The **schemaless-by-default** philosophy is applied at the boundary: schema mismatches return warnings in the response, never reject writes.

**Backlog source:** SPEC-013 (promoted per the Spec Intake Rule).

## Motivation

Eight specs are implemented with no REST surface — only the SSE stream (SPEC-004). The frontend (G6/Bit), pipeline batches (SPEC-011), audit queries (SPEC-012), and the registry console (SPEC-009) all need HTTP endpoints. This spec is a thin composition layer: no new domain logic, just wiring + the principal/error/warning contract.

## Requirements

### Requirement: Entity & Property-Set CRUD

The system SHALL expose REST endpoints to create entities, save property sets (with optional correlation id), load property sets, resolve a scope chain, and revert to a commit.

#### Scenario: Create and resolve via HTTP
- **GIVEN** an authenticated request
- **WHEN** an entity is created and a property set saved
- **THEN** resolving the scope chain returns the effective property set

### Requirement: Moderation Surface

The system SHALL expose change-request submission, decision, apply, promotion, and request listing.

#### Scenario: Submit and decide via HTTP
- **GIVEN** a change request submitted via the API
- **WHEN** a reviewer decides and applies
- **THEN** the property set is updated and the decision is auditable

### Requirement: Audit Queries

The system SHALL expose entity and actor audit queries (SPEC-012 projections).

#### Scenario: Entity audit via HTTP
- **GIVEN** activity on an entity
- **WHEN** the entity audit endpoint is queried
- **THEN** commit-ordered entries with actor/action/commit are returned

### Requirement: Registry Surface

The system SHALL expose namespace listing and `additionalType` validation as a **warning** outcome (schemaless philosophy — never rejects).

#### Scenario: additionalType mismatch warns
- **GIVEN** an entity with an unregistered `additionalType` IRI
- **WHEN** validated via the API
- **THEN** a warning is returned and no write is rejected

### Requirement: Streaming Mount

The system SHALL mount the SPEC-004 SSE endpoint on the same application.

#### Scenario: Stream served by the API
- **GIVEN** a running API server
- **WHEN** a client connects to `/events`
- **THEN** live knowledge events are streamed

### Requirement: Principal Contract

The system SHALL accept a principal identifier on each request (header in v1) and use it as commit `author` and ledger actor.

#### Scenario: Principal is recorded
- **GIVEN** a request with principal `pipeline-x`
- **WHEN** a write is performed
- **THEN** the commit author and audit actor are `pipeline-x`

## Acceptance Criteria

- AC-1: Given an entity + property set created via HTTP, when the chain is resolved, then the effective set is returned. ✅
- AC-2: Given a change request submitted via HTTP, when decided + applied, then the property set updates and the decision appears in the audit. ✅
- AC-3: Given entity activity, when the audit endpoint is queried, then commit-ordered entries are returned. ✅
- AC-4: Given an unregistered `additionalType`, when validated via the API, then a warning is returned (no rejection). ✅
- AC-5: Given a running API, when a client connects to `/events`, then live events are streamed. ✅
- AC-6: Given a request with a principal header, when a write happens, then commit author + audit actor equal the principal. ✅

**Implementation notes (2026-08-07):** `create_entity_as(kind, author)` added (principal → commit author); registry gains `list_namespaces`; `validate-additional-type` converts `TypeNotFound` into an `api:Warning` (schemaless philosophy at the boundary); stream mounted via `FromRef` state plumbing; tests serialize against the shared server (2-worker contention).

## Technical Design

### Crate: `backend/crates/api` (axum)

```text
backend/crates/api/
  main.rs            # bootstrap: repository + router + serve
  auth.rs            # X-Principal extractor (v1); SPEC-003 wiring documented
  error.rs           # JSON-LD-ish error contract { "@type": "api:Error", ... }
  routes/
    entities.rs      # POST /entities, POST /entities/{id}/property-sets, GET /entities/{id}/resolve
    moderation.rs    # POST /requests, POST /requests/{id}/decide, POST /requests/{id}/apply, POST /promotions
    audit.rs         # GET /audit/entities/{id}, GET /audit/actors/{actor}
    registry.rs      # GET /namespaces, POST /validate-additional-type (warning outcome)
    stream.rs        # mounts stream_api router at /events
```

- Reuses `terminusdb-repository` (entities/property sets/audit), `schema-registry` (namespaces), `stream-api` (SSE)
- Moderation: v1 = thin wrapper over ledger docs (SPEC-012) + repository writes; the in-memory `ModerationLedger` state machine remains domain-side (SPEC-002) until the application layer adopts it fully
- Warning contract: `{ "@type": "api:Warning", "api:warnings": [...] }` alongside the normal payload (philosophy)
- Error contract: `{ "@type": "api:Error", "api:message": ... }`

### Auth (v1 risk)
- `X-Principal: <id>` header; no verification (documented R-18). Sessions/roles from SPEC-003 + verified authors = SPEC-014 (auth) follow-up.

## Test Plan

- [ ] Integration: entity CRUD + resolve round-trip (AC-1)
- [ ] Integration: moderation submit/decide/apply + audit visibility (AC-2)
- [ ] Integration: audit endpoint (AC-3)
- [ ] Integration: additionalType warning (AC-4)
- [ ] Integration: /events mounted (AC-5)
- [ ] Integration: principal recorded in author/actor (AC-6)

## Open Risks

| ID | Risk | Impact | Mitigation |
|----|------|--------|------------|
| R-18 | Unverified principal header (spoofable authors) | High — audit integrity | Documented; SPEC-014 (auth) mandatory before production |
| R-19 | In-memory ModerationLedger vs ledger docs — two views | Medium | v1 API uses ledger docs (SPEC-012) as the source for queries; state machine adoption later |
| R-20 | No rate limiting/quotas | Medium | Defer to SPEC-014 or SPEC-007 gates |

## Relationship to Other Specs

- SPEC-001/002/003: composed surfaces; SPEC-003 principals deferred to SPEC-014
- SPEC-004: SSE mounted; SPEC-009: registry warnings; SPEC-012: audit queries
- SPEC-011 (planned): batch API lands on this surface; SPEC-014 (planned): real auth


---

## ARCHIVED (2026-08-07)

- **Verification commit:** `bae983a`
- **Evidence:** backend/crates/api/tests/spec_013_api.rs
- **ACs:** 6/6 verified green against real TerminusDB 12.1 (TerminusDBServer pattern)
- **Status change:** Implemented → Archived. Re-check (spec-vs-code convergence) if touched by future work.
