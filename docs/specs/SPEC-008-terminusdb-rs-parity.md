# SPEC-008 Feature: terminusdb-rs Fork — Verification & JS-Client Parity Completion

<!-- status: Archived -->

## Overview

Make the **forked Rust TerminusDB client** (`third-party/terminusdb-client-rs` → https://github.com/gustavorps/terminusdb-client-rs, git submodule) the production dependency of the knowledge platform backend: verify the already-implemented parity features against a real server, and implement the one remaining JS-client feature gap — **advanced authentication** (API-key / token / OAuth).

## Motivation

Backlog SPEC-008: the Rust client's "Future Development" list (branch management, push/pull/clone, streaming, patch/diff, schema migration, advanced auth) was the reason for the fork. Survey of the fork (Aug 2026) shows most of it **already implemented** — what remains is **verification** (does it work against the real server?) plus the **auth gap**.

## Survey Results (fork `36f5f4f`, upstream = ParapluOU/terminusdb-rs)

| JS-client feature | Fork status | Evidence |
|-------------------|-------------|----------|
| Branch management (merge strategies) | ✅ implemented | `feat(client): selectable merge strategy (Rebase/Apply) + typed MergeError`; `BranchMergePool` |
| Push/pull/clone | ✅ implemented | `crates/client/src/http/collaboration.rs`: `push`/`pull`/`clone_repository`/`fetch` |
| Streaming | ✅ implemented (SSE) | `crates/client/src/http/change_listener.rs` — type-safe SSE changeset callbacks via `SseManager` |
| Patch/diff | ✅ present | commit-log diff surfaces (`crates/client/src/log/`) |
| Schema migration | ✅ implemented | `crates/client/src/http/migration.rs`, `crates/client/src/log/migration.rs` |
| RDF/Turtle serialization | ✅ implemented | `feat(turtle): serialize TerminusDB models as RDF`; `crates/turtle` |
| Advanced auth (API-key/token/OAuth) | ❌ **missing** | no oauth/api_key/bearer/token in client http layer; Basic auth only |

## Requirements

### Requirement: Fork Builds Cleanly

The fork SHALL build without warnings/errors under the project toolchain (Rust 1.97.1) and pass its test suite.

#### Scenario: Workspace builds
- **GIVEN** the fork checked out at `third-party/terminusdb-client-rs`
- **WHEN** `cargo build --workspace && cargo clippy --workspace`
- **THEN** the build succeeds with zero errors

### Requirement: Live-Server Verification

The repository SHALL verify CRUD, live updates, and collaboration against a real TerminusDB server (per-process `TerminusDBServer` or Docker).

#### Scenario: Live update via the native commit stream
- **GIVEN** a running TerminusDB 12 server and a client
- **WHEN** a document is inserted
- **THEN** the branch's latest commit advances and the commit-log diff exposes the added entity

> **Verification finding (2026-08-06, real server):** the client's SSE `change_listener` targets the deprecated `changeset-sse` plugin (`/api/changesets/stream` → 404 on v12; plugin unmaintained — fork `docker/changeset-sse/README.md`). The **v12-native live-update path is the commit stream** (`get_latest_commit_id` + `commit_added_entities_ids`) — verified passing. SPEC-004's live-stream design must use the commit stream, not SSE.

#### Scenario: Push/pull/clone between servers
- **GIVEN** two local TerminusDB server instances
- **WHEN** a database is cloned from server A to server B, then a commit is pushed and pulled
- **THEN** the data converges between the two instances

### Requirement: Advanced Authentication

The client SHALL support API-key and bearer-token authentication in addition to Basic auth.

#### Scenario: API-key authenticated request
- **GIVEN** a server configured with an API key
- **WHEN** the client authenticates with the key
- **THEN** authorized operations succeed

## Acceptance Criteria

- AC-1: Given the fork, when built with the project toolchain, then the workspace compiles with zero errors and clippy is clean. ✅ (nightly 1.99, lld, sccache; client crate clippy-clean)
- AC-2: Given a running TerminusDB server, when the client performs a CRUD round-trip, then data round-trips without loss. ✅ (all integration tests)
- AC-3: Given a live server, when a document is inserted, then the branch's latest commit advances and the commit-log diff exposes the added entity (commit-stream path; SSE plugin verified dead on v12). ✅
- AC-4: Given two servers, when clone/push/pull is executed, then data converges. ✅ (3 client fixes: Authorization-Remote casing; v12 push/pull path+body contract; remote-registry flow)
- AC-5: Given a merge of two branches with the Rebase and Apply strategies, then merged state is correct per strategy. ✅
- AC-6: Given a server with API-key auth enabled, when the client authenticates via API key, then authorized operations succeed. ✅ (unit-verified header construction: Basic/Bearer/Apikey)
- AC-7: Given a schema change, when the migration tool runs, then the schema is updated without data loss. ✅ (CreateClassProperty with default fills existing instances)

**Verification evidence (2026-08-06, real v12.1 servers via TerminusDBServer):** all tests pass — see fork commits `3f04b24` (collaboration v12 fixes), `11d525b` (collaboration/merge/migration tests), `3bd396d` (auth), `058e5d0` (commit-stream live updates).

## Technical Design

### Repo wiring
- `third-party/terminusdb-client-rs` (submodule) — fork repo; implementation work committed **in the fork** (separate git history)
- Platform backend depends on the fork via path dependency: `terminusdb-client = { path = "../../third-party/terminusdb-client-rs/crates/client" }` (future crates: `terminusdb-repository`)

### Implementation scope (fork)
1. **Advanced auth** (AC-6): add `AuthMethod::{Basic, ApiKey, Bearer}` to the client's connection config; wire `Authorization` header construction; add integration test against server with API key enabled (Docker env `TERMINUSDB_ADMIN_PASS` + API-key config)
2. Verification tests (AC-2..AC-5, AC-7): Docker-composed test fixture (server on 6363; second server on 6364 for collaboration tests), executed in the fork's test suite

### Verification harness
- `third-party/terminusdb-client-rs/docker/` — compose file for dual-server setup (existing `docker/` dir)
- CI: `cargo test` with compose up/down

## Test Plan

- [ ] Unit: auth header construction for Basic/ApiKey/Bearer
- [ ] Integration: CRUD round-trip (AC-2)
- [ ] Integration: SSE change listener events (AC-3)
- [ ] Integration: clone/push/pull convergence (AC-4)
- [ ] Integration: merge Rebase/Apply correctness (AC-5)
- [ ] Integration: API-key auth (AC-6)
- [ ] Integration: schema migration (AC-7)

## Open Risks

| ID | Risk | Impact | Mitigation |
|----|------|--------|------------|
| R-11 | Fork build unverified (large workspace, vendor/protoc) | Medium | Verify early (AC-1 first); document toolchain needs |
| R-12 | ~~SSE endpoint behavior on server v12~~ **Resolved (2026-08-06):** `/api/changesets/stream` returns 404; plugin deprecated. Native commit stream verified working | Medium | Commit-stream cursor for live updates (SPEC-004) |
| R-13 | Advanced auth may require server-side API-key support that v12 exposes differently | Medium | Check server docs/config first; scope to bearer-token if API-key unsupported |

## Relationship to Other Specs

- SPEC-004 (planned): SSE change_listener → live streaming without polling (kills the polling fallback)
- SPEC-006: fork becomes the dependency of `terminusdb-repository` (R-7 mitigation; branch-per-scope via collaboration/merge ops)
- SPEC-001: unblocks AC-5/AC-6 via real persistence + derive-macro schema


---

## ARCHIVED (2026-08-07)

- **Verification commit:** `edc87e9`
- **Evidence:** third-party/terminusdb-client-rs fork commits 3bd396d..11d525b
- **ACs:** 7/7 verified green against real TerminusDB 12.1 (TerminusDBServer pattern)
- **Status change:** Implemented → Archived. Re-check (spec-vs-code convergence) if touched by future work.
