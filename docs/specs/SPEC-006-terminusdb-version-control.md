# SPEC-006 Feature: TerminusDB Git-Like Version Control & Temporal Architecture

<!-- status: Archived -->

## Overview

Adopt TerminusDB (v12, DFRNT) as the **system of record** for the knowledge platform, leveraging its Git-for-data capabilities — commits, branches, diff, time-travel queries, JSON+RDF dual model, and Allen-interval temporal reasoning — to replace in-memory mechanisms from SPEC-001/002 and give every graph primitive full version control, provenance, and temporal querying.

**R-1 is now verified** (2026-08-06): GitHub README [Source: External] github.com/terminusdb/terminusdb + hands-on local Docker run (image `terminusdb/terminusdb-server`, port 6363) + Rust client review [Source: External] github.com/ParapluOU/terminusdb-rs.

## Motivation

Brainstorm SPEC-006 (backlog): *"Git-like version control combining JSON and RDF with high precision temporal reasoning to improve architecture and design patterns."* SPEC-002 keeps per-scope `history: Vec<PropertySet>` in memory (R-3); SPEC-002's promotion ladder hand-rolls what TerminusDB does natively (branches/merges/commits); provenance is a bespoke record where commit metadata would do. TerminusDB's commit graph + time-travel makes the moderation process auditable by construction and enables `resolve(entity, scope, at)` — temporal resolution across scope layers.

## Requirements

### Requirement: Single Store of Record

The system SHALL persist all domain state (entities, scoped property sets, scope hierarchy, change requests, provenance) in TerminusDB via the Rust client (`terminusdb-client`), replacing in-memory storage. PostgreSQL remains auth/sessions only.

#### Scenario: Entity persists across restarts
- **GIVEN** an entity created and property set applied via the repository
- **WHEN** the process restarts and re-reads
- **THEN** the entity and its scoped property sets are present with versions intact

### Requirement: Commits as the Audit Trail

Every write SHALL be a TerminusDB commit carrying `author` (principal from SPEC-003) and `message`; the commit graph SHALL serve as the provenance/audit record (superseding the bespoke `Provenance` struct where equivalent).

#### Scenario: Change request apply is a commit
- **GIVEN** an approved change request (SPEC-002)
- **WHEN** it is applied
- **THEN** one TerminusDB commit records the new version with author = reviewer, message = request id

### Requirement: Temporal Resolution

The system SHALL support time-travel queries: `resolve(entity, scope, at: commit-or-time)` returns the effective property set as of that point, using TerminusDB time-travel queries and (for interval semantics) Allen-interval temporal reasoning.

#### Scenario: As-of resolution
- **GIVEN** an entity whose org set changed at commit N
- **WHEN** resolved as of commit N-1
- **THEN** the pre-change property set is returned

### Requirement: Promotion as Branch/Merge (design mapping)

The system SHALL map the promotion ladder (SPEC-002) onto TerminusDB git semantics: child scope work on a branch; promotion = merge request → merge; adoption = fast-forward.

#### Scenario: Promotion maps to merge
- **GIVEN** a workspace-scope branch with a promoted version
- **WHEN** the promotion is approved and merged
- **THEN** the parent scope's main branch contains the merged version with a merge commit

### Requirement: JSON + RDF Dual Model

The system SHALL store scoped property sets as JSON documents (via `terminusdb-schema` derive-macro models — closing SPEC-001 R-2 codegen) and graph topology (edges, combos, hierarchy) as linked documents / RDF triples queryable via WOQL/GraphQL.

#### Scenario: Schema derives from Rust models
- **GIVEN** Rust models annotated with `#[derive(TerminusDBModel)]`
- **WHEN** the schema is inserted
- **THEN** data validates against the schema and graph links resolve across documents

## Acceptance Criteria

- AC-1: Given persisted entities, when the process restarts and re-reads, then entities and scoped property sets are present with versions intact. ✅
- AC-2: Given an approved change request, when applied, then a TerminusDB commit exists with author = reviewer and message = request id. ✅
- AC-3: Given an entity whose org set changed at commit N, when resolved as of N-1, then the pre-change set is returned. ✅
- AC-4: Given a workspace branch with a promoted version, when approved and merged, then the parent branch contains the merged version. ✅ (fork merge Rebase/Apply verified; v2 branch-per-scope mapping)
- AC-5: Given Rust `TerminusDBModel` models, when schema is inserted, then data validates and graph links resolve. ✅ (fork `with_db_schema` pattern)
- AC-6: Given domain state, when persisted and re-loaded into the domain API, then invariants (versions, ladder resolution) hold. ✅

**Implementation note (2026-08-06):** property sets are stored as immutable versioned documents (`PS:{entity}:{instance}:v{version}`); each save is a commit with author + `{message}|ps:...` token; `resolve_at` reconstructs as-of state from the commit log (the fork client's `ref_commit` time-travel read is not wired). sys:JSON fields require `unfold: true` to materialize (content-addressed refs otherwise).

## Technical Design

### Crate structure (backend workspace)

```text
backend/crates/
  data-graph/        # pure domain (existing — unchanged)
  terminusdb-repository/   # NEW: persistence adapter (hexagonal "infrastructure" port)
    - repository.rs        # implements domain repository port
    - models.rs            # #[derive(TerminusDBModel)] maps (closes SPEC-001 R-2)
    - commits.rs           # author/message mapping, commit-id tracking
    - temporal.rs          # as-of resolution via time-travel queries
  api/                     # future: REST (axum) application layer
```

### Key verified facts (drives design)

| Fact | Source | Design impact |
|------|--------|---------------|
| Commits per update; diff as patches; push/pull/clone | GitHub README + local run (2 commits verified) | Audit trail by construction |
| Time-travel queries (any state at any commit) | GitHub README | `resolve(... at:)` temporal AC-3 |
| Allen Interval Algebra, ISO8601, arbitrary-precision decimal (v12) | GitHub README | Temporal reasoning for knowledge lifecycles |
| JSON Git-for-Data (JSON/JSON-LD/XML/Turtle) + schema control | GitHub README | Scoped property sets as versioned JSON |
| WOQL datalog, GraphQL, REST with deep-link discovery | GitHub README | Graph queries for topology |
| Rust client: derive-macro schema, ORM, WOQL2, commit-id tracking, BranchSpec | ParapluOU repo README | AC-5 codegen story (R-2); `insert_instance_with_commit_id` |
| **Client now verified end-to-end (SPEC-008, Aug 2026):** clone/push/pull convergence, merge Rebase/Apply, schema migration, commit-stream live updates, bearer/api-key auth — all green against real v12.1 servers | Fork commits 3bd396d..11d525b | R-7 substantially retired; collaboration flow documented (remote registration + fetch-before-push) |
| SSE plugin endpoint dead on v12 (`/changesets/stream` → 404) | SPEC-008 verification | Live updates = commit stream, not SSE |
| Rust client 8 stars / 1 fork; missing branch mgmt, streaming, diff/patch (JS-client parity pending) | ParapluOU repo README ("Future Development") | ⚠️ coupling risk — abstract repository port so client gaps don't block domain |
| Server runs in Docker, Basic auth, JSON-LD API errors | Local run 2026-08-06 | Dev-loop viability |
| Raw WOQL JSON via REST is finicky; GraphQL/REST simpler | Local run 2026-08-06 | Prefer Rust client WOQL2 builder / ORM over raw WOQL JSON |
| Live subscriptions/streaming: NOT found in server/client | Local run + repos | SPEC-004 must use polling (or TerminusDB's commit-log polling as cursor) |

### Versioning mapping
- Domain `version` (SPEC-001 monotonic per scope) ↔ TerminusDB commit sequence per branch
- `history` (SPEC-002 R-3) → replaced by TerminusDB commit graph (diff/time-travel)
- `Provenance` (SPEC-002) → commit metadata (author/message), kept as convenience projection
- Scope instances (SPEC-003) → one TerminusDB branch per scope instance (target architecture); v1 may keep main-branch + per-scope keyed documents

### Migration path (v1 → target)
- v1: single `main` branch; all scopes as documents with scope keys (matches current domain API)
- v2: branch-per-scope promotion mapping (AC-4) once Rust client branch ops mature (or via REST)

## Test Plan

- [ ] Integration: repository round-trip through real TerminusDB container (AC-1, AC-6) — unblocks SPEC-001 AC-6
- [ ] Integration: apply creates commit with author/message (AC-2)
- [ ] Integration: as-of resolution returns pre-change state (AC-3)
- [ ] Integration: schema derive + validation + graph link resolution (AC-5) — unblocks SPEC-001 AC-5
- [ ] Unit: domain invariants hold after persist/reload cycle (AC-6)
- [ ] CI: Docker-composed TerminusDB test fixture

## Open Risks

| ID | Risk | Impact | Mitigation |
|----|------|--------|------------|
| R-7 | Rust client maturity (8 stars, 1 fork; branch/streaming/diff parity pending) | High — core dependency | Abstract repository port (hexagonal); fall back to REST/GraphQL calls for missing ops; pin versions; track upstream |
| R-8 | Raw WOQL via REST is fragile | Medium | Use Rust client WOQL2 builder/ORM; integration tests guard queries |
| R-9 | Branch-per-scope (v2) needs client branch ops that may lag | Medium | Keep v1 main-branch model; revisit on upstream maturity |
| R-10 | Commit-per-write latency (fsync) at high stream rates | Medium | Batch writes per commit where semantics allow; monitor SPEC-004 rates |

## Relationship to Other Specs

- SPEC-001: closes R-1 (persistence), R-2 (derive-macro schema); unblocks AC-5/AC-6
- SPEC-002: replaces `history` (R-3), maps promotion to merge (R-9)
- SPEC-003: principals become commit `author`
- SPEC-004 (planned): commit-log polling as the live-stream cursor
- Backlog SPEC-005 (merge): TerminusDB diff/merge gives native conflict detection

---

## MODIFIED Requirements (delta — 2026-08-07)

- **MODIFIED — Message token format:** commit-message tokens are `{user-message}|ps:{entity}:{instance}:{scope}:v{version}` (scope added by SPEC-004). `resolve_at` parses both the 3-part legacy and 4-part current forms.
- **MODIFIED — Time-travel mechanics:** v1 as-of resolution is reconstructed from the commit log (fork client `ref_commit` reads not wired); an explicit `get_document_as_of` client feature is a candidate fork follow-up.


---

## ARCHIVED (2026-08-07)

- **Verification commit:** `26cefa5`
- **Evidence:** backend/crates/terminusdb-repository/tests/spec_006_repository.rs
- **ACs:** 6/6 verified green against real TerminusDB 12.1 (TerminusDBServer pattern)
- **Status change:** Implemented → Archived. Re-check (spec-vs-code convergence) if touched by future work.
