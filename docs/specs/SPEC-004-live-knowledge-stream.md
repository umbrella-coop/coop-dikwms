# SPEC-004 Feature: Live Knowledge Stream via the TerminusDB Commit Stream

<!-- status: Archived -->

## Overview

Expose knowledge changes as a typed **domain event stream** over the TerminusDB commit stream (verified in SPEC-008: the SSE plugin is dead on v12; the native commit stream works), delivered to clients via an **SSE endpoint**. v1 emits knowledge events only: `EntityCreated` and `PropertySetSaved` (covers property changes, promotions, combo regroupings — all are property-set saves). Moderation-action events (PostgreSQL, PG19 SQL/PGQ) are a separate future spec.

**Backlog source:** SPEC-004 (promoted per the Spec Intake Rule).

## Motivation

The original problem statement requires *live stream data* for nodes and combos. SPEC-008 proved the mechanism: every write is a commit; the commit log + message tokens (`ps:{entity}:{instance}:v{version}`) reconstruct domain events without polling the database for deltas. G6 consumers need event-level updates (redraw only what changed), and the activity stream (SPEC-002 idea) needs the change-request lifecycle as events later.

## Requirements

### Requirement: Commit-Cursor Polling

The system SHALL expose `poll_events(repo, cursor) -> (Vec<DomainEvent>, new_cursor)`: given a commit identifier (or genesis), return all domain events from commits strictly newer than the cursor, in chronological order, advancing the cursor monotonically.

#### Scenario: New commits yield exactly the new events
- **GIVEN** a repository with commits up to commit N
- **WHEN** `poll_events` runs with cursor N
- **THEN** it returns no events and cursor stays N
- **AND** after a property-set save (commit N+1), `poll_events` returns exactly the `PropertySetSaved` event

#### Scenario: Genesis cursor replays history
- **GIVEN** a repository with several commits
- **WHEN** `poll_events` runs with the genesis cursor
- **THEN** all past events are returned in commit order

### Requirement: Event Decoding

The system SHALL decode commit messages into typed events: `EntityCreated { entity_id, kind, commit }` (from `ent:{id}:{kind}` tokens) and `PropertySetSaved { entity_id, instance_id, scope, version, commit }` (from `ps:` tokens). Messages without recognized tokens SHALL be skipped without failing the stream.

#### Scenario: Unknown messages are skipped
- **GIVEN** a commit log containing unrecognized messages
- **WHEN** `poll_events` runs
- **THEN** the recognized events are returned and the unknown messages are ignored

### Requirement: SSE Delivery

The system SHALL expose an SSE endpoint `GET /events?cursor=<commit>` that streams events as commits land, polling the commit log on a fixed interval. The response SHALL include the advancing cursor (`id:` field per event) so clients can reconnect without loss.

#### Scenario: Client receives events as they land
- **GIVEN** a connected SSE client with a cursor
- **WHEN** a property set is saved
- **THEN** the client receives the `PropertySetSaved` event within one poll interval

## Acceptance Criteria

- AC-1: Given a repository with past commits, when `poll_events` runs from genesis, then all events are returned in commit order. ✅
- AC-2: Given a cursor at commit N, when a property set is saved, then `poll_events` returns exactly the new event and advances the cursor. ✅
- AC-3: Given multiple new commits, when `poll_events` runs, then events appear once each, in commit order (monotonic cursor). ✅
- AC-4: Given an SSE client connected with a cursor, when a property set is saved, then the event arrives within one poll interval. ✅
- AC-5: Given unknown commit messages, when `poll_events` runs, then they are skipped without failing the stream. ✅

**Implementation notes (2026-08-07):** `ps:` tokens extended with the scope (`ps:{entity}:{instance}:{scope}:v{version}`); `resolve_at` parses both 3-part legacy and 4-part current tokens. `EntityCreated` events from `create-entity|ent:{id}:{kind}` tokens. SSE events carry the originating commit as `id:` (lossless reconnect).

## Technical Design

### Repository addition (`terminusdb-repository`, `stream.rs`)
```rust
pub enum DomainEvent {
    EntityCreated { entity_id: Uuid, kind: String, commit: String },
    PropertySetSaved { entity_id: Uuid, instance_id: Uuid, scope: String, version: u64, commit: String },
}

pub struct CommitCursor(pub String);   // "" = genesis

pub async fn poll_events(repo: &Repository, cursor: &CommitCursor) -> anyhow::Result<(Vec<DomainEvent>, CommitCursor)>
```
- `create_entity` gains a message token: `ent:{entity_id}:{kind}` (backwards compatible — existing tests unaffected)
- `poll_events`: fetch `log()` (newest-first), walk from the newest commit down to the cursor; collect tokens chronologically; skip unknown messages (AC-5); new cursor = newest commit identifier

### New crate: `stream-api` (axum SSE transport)
```text
backend/crates/stream-api/
  main.rs / lib.rs: Router with GET /events
  sse.rs:           poll loop (tokio interval, default 500ms), cursor from query param,
                    SSE events with `id: <cursor>` + `data: <json DomainEvent>`
```
- Deps: axum 0.8, tokio (time/sync), tokio-stream, futures, terminusdb-client, terminusdb-repository
- One poll loop **per connection** in v1 (simple; broadcast channel later if load demands)
- Client contract: `GET /events?cursor=<commit>` → stream; reconnect with the last `id:` received

### Out of scope (future specs)
- Moderation-action events (PostgreSQL/PGQ)
- Change-request lifecycle events (needs ledger persistence)
- Broadcast/multi-consumer fan-out, backpressure policies

## Test Plan

- [ ] Unit: token decoding (ent:/ps:), unknown-message skipping (AC-5)
- [ ] Integration (real server): genesis replay in order (AC-1); cursor advance + exactly-new events (AC-2, AC-3)
- [ ] Integration: SSE endpoint delivers events within one poll interval (AC-4)
- [ ] Integration: reconnect with cursor resumes without loss (AC-4 extension)

## Open Risks

| ID | Risk | Impact | Mitigation |
|----|------|--------|------------|
| R-14 | Poll interval latency (500ms) vs true push | Low | Interval config; broadcast channel later |
| R-15 | Commit log growth — replay from genesis grows unbounded | Medium | Client cursor persistence; snapshot endpoint later |
| R-16 | Per-connection polling at scale | Low | v1 constraint documented; channel fan-out later |

## Relationship to Other Specs

- SPEC-008: verified commit-stream mechanism (foundation)
- SPEC-006: repository + message tokens consumed here; `create_entity` token addition
- SPEC-002: activity stream (change-request lifecycle events) later, via ledger persistence
- SPEC-003: scope-scoped subscriptions later (filter events by instance in chain)
- Future: moderation-action events (PostgreSQL), API layer consolidation


---

## ARCHIVED (2026-08-07)

- **Verification commit:** `a5a8dbe`
- **Evidence:** backend/crates/terminusdb-repository/tests/spec_004_stream.rs + stream-api/tests/spec_004_sse.rs
- **ACs:** 5/5 verified green against real TerminusDB 12.1 (TerminusDBServer pattern)
- **Status change:** Implemented → Archived. Re-check (spec-vs-code convergence) if touched by future work.
