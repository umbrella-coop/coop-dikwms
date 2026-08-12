# Branch: `speed-run-1/git-domain`

**Git-Domain Speed Run — end-to-end proof of the DIKW platform** (SPEC-027, implemented 2026-08-11).

## Context

This branch carries the first fully functional **backend + frontend** slice of the platform, proving it end-to-end across all four DIKW layers on a real dataset — the git history of `terminusdb/terminusdb` (12-month window, fixed cutoff `2025-08-11` for reproducible imports):

- **Data** — `git-importer` crate walks a local clone via `git2` and persists commits/authors as generic graph primitives through `POST /bulk-entities` (server-side idempotency via `dedupe_key`, per-batch checkpoints, retries + circuit breaker).
- **Information** — G6 explorer UI (`http://localhost:3100`): timeline slider, author filter chips, node drawer with property-set history (resolve_at time-travel), registry-resolved `GraphNodeShape` distinguishing git entities (SPEC-027 AC-9).
- **Knowledge** — `git.v1` namespace registered in the schema-registry; organic violations (merge commits, empty messages) surface as `api:warnings`, never rejections (schemaless warn-not-fail).
- **Wisdom** — insight post-pass computes ownership concentration, stale-knowledge zones, and bus-factor risk per directory, stored as property sets on a per-repo insight entity.

## What's here

- `backend/crates/git-importer` — import binary + Phase C insight post-pass
- `backend/crates/api` — `POST /bulk-entities`, SSE `/events?cursor=` live materialization
- `backend/crates/schema-registry` — `git.v1` namespace registration
- `frontend/dikwms/` — G6 git explorer, `use-data-graph-sse` hook
- `examples/git-codebase-1/` — clone/import/verify/serve runbook, deterministic with the fixed cutoff

## Run it

```sh
docker compose up -d terminusdb              # TerminusDB on :6363
cd backend && env RUSTUP_TOOLCHAIN=nightly cargo run -p api   # API on :8080
cd frontend && bit run app -p 3100           # UI on :3100
```

## Status

Phase A–C implemented; SPEC-027 records 8/11 AC PASS, 3 partial (implementation record `docs/specs/SPEC-027-git-domain-speed-run.md`). Live-materialization E2E suite real-browser (Playwright, `e2e/run-e2e.mjs`).

**Do not merge into `main`** until the 3 partial ACs are closed.
