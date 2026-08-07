# Brainstorm Report: Graph Conductor — Roadmap Fit (SPEC-011 input)

> Generated: 2026-08-07 · Source: reconurge/flowsint#133 RFC (gustavorps) · Session: mapping RFC mechanisms onto the coop-graph-network stack

## Verdict
**The RFC validates our architecture: most coordination mechanisms already exist in our stack under different names.** The genuinely new work is the *batch/pipeline surface*: ingestion API, idempotency keys, fast-track approval, revert, DLQ, pipeline identity, MCP integration. Recommended: promote **SPEC-011 = Batch Ingestion & Pipeline Coordination**, reusing existing machinery rather than building a new conductor layer.

## Mechanism-by-mechanism mapping

| RFC mechanism | Our equivalent | Status | Notes |
|---------------|----------------|--------|-------|
| Command queue | **Change requests (SPEC-002)** | ✅ exists | Batches map to fast-track-tier requests |
| Region locking (Redis, pessimistic) | **TerminusDB branches (optimistic)** | ⚠️ design choice | RFC locks; we *isolate* — agents/pipelines work on branches, merge with conflict detection (SPEC-008 verified Rebase/Apply). Trade-off documented below |
| Version vectors (OCC) | **Property-set versions + version guard (SPEC-001/002)** | ✅ exists | `SetError::VersionOutOfOrder` == VERSION_CONFLICT |
| Event sourcing + revert | **Commit log (SPEC-006) + `resolve_at`** | ✅ exists | TerminusDB commits ARE the event store; revert = time-travel/reset |
| Idempotency registry | **Idempotent apply (SPEC-002 AC-4)** | 🔶 partial | Entity-level apply is idempotent; **batch-level idempotency keys are new** |
| Role-scoped permissions | **Policy/ACL (SPEC-003)** | ✅ exists | Map agent roles onto `Permission::{Submit,Decide,...}` |
| DLQ | — | ❌ new | Failed/rejected batches need visibility + retry |
| Anti-corruption layer | **Schemaless philosophy + registry warnings (SPEC-009)** | ✅ exists | We WARN on schema mismatch; RFC validates hard — our stance is softer by design |
| WebSocket event stream | **SSE commit stream (SPEC-004)** | ✅ exists | Pipeline progress events ride the same stream |

## Design decisions for SPEC-011

**D1 — Concurrency model: optimistic branches, not locks.** RFC's pessimistic region locks protect a single mutable store. We have versioned, branchable storage: pipelines write on their own branch (or fast-track change requests), conflicts surface as version mismatches/merge conflicts at promotion. Adopt RFC's **claim-before-work** spirit (declare `expected_versions`) without building a Redis lock layer.

**D2 — Batch semantics: one fast-track change request per batch.** Trusted pipelines (org-configured, SPEC-003 role) get auto-approval (fast-track tier from the platform brainstorm); untrusted pipelines land in the moderation queue as one reviewable unit. Idempotency key per batch → dedupe retries (RFC §7 key convention: deterministic hash of pipeline+intent).

**D3 — Revert is time-travel.** Expose `resolve_at`/commit reset as a revert API (`revert batch B` = restore pre-batch state). TerminusDB's git model gives this for free — no compensating commands needed (RFC §6.3 solved differently).

**D4 — Pipeline identity.** Pipeline = SPEC-003 principal with a role; commit `author` = pipeline id (RFC's agent_id/agent_role map directly). Audit = commit log (already).

**D5 — Tech surface (in priority order):** 1) HTTP batch API (future API layer — same spec), 2) **MCP server** (the fork already has `crates/mcp-server` — agents can drive the platform natively), 3) Airflow operator later.

## Scope for SPEC-011 (promotion recommendation)
- Batch change requests with fast-track tiers + idempotency keys (D1/D2)
- Revert API via time-travel (D3)
- Pipeline identity + role permissions (D4)
- DLQ: failed/rejected batch visibility + retry
- MCP server surface (D5)
- **Out of scope:** Redis lock layer (D1), Neo4j-style version migrations (our versioning is schema-native), Celery queue (replaceable by any pipeline tech)

## Deferred
- Cross-technology SDKs (Airflow operator) until the HTTP API proves out
- Multi-pipeline conflict *merge* UX (TerminusDB merge exists; conflict-resolution UI later)

## Next Steps
- [ ] Promote SPEC-011 from backlog with the scope above
- [ ] Note in SPEC-002: fast-track tier policy (batch auto-approval)
- [ ] Verify: does the fork's mcp-server crate expose document insert commands? (gates D5)
