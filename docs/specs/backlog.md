# Spec Backlog

Planned specs not yet created. Add requirements captured during development here so they are not lost.

> **Intake rule (added 2026-08-07):** every spec source feeds this queue — brainstorm outcomes, user requests, dependency findings, and spec-relationship references. A spec is created from the queue, not on demand.

## SPEC-004 (planned): Live Streaming via the Commit Stream

**Source:** original problem statement ("live stream data") + brainstorm idea #5 (CDC + cursor, Agg 4.1) — referenced as "planned" in SPEC-001/002/006/008 relationships but never created (gap).

**Status:** **fully de-risked by SPEC-008** — SSE plugin endpoint is dead on v12 (404); the **native commit stream is verified working** (commit advances + `commit_added_entities_ids` diff).

**Design sketch:**
- Live updates = commit-log cursor over `terminusdb-repository` (poll/`log_iter` diff per cursor position)
- Domain events: node created, property promoted, combo regrouped, change-request lifecycle (feeds SPEC-002 activity stream)
- Client reconciliation via cursor (no redraw storms — G6 consumes domain events, not raw deltas)
- PostgreSQL events for moderation actions (PG19 SQL/PGQ — verify) vs TerminusDB commit stream for knowledge changes — boundary decision for the spec

## SPEC-016 (planned): Programmatic API Docs & OpenAPI Generation (mdBook)

**Requirement (captured 2026-08-07):** Programmatic API documentation and **OpenAPI spec generation**, integrated with **mdBook** as the documentation site.

**Design elements to resolve in the spec:**
- **OpenAPI generation:** `utoipa` derive macros on the SPEC-013 axum handlers (path/response schemas) — or alternative generators; spec emitted as a build artifact (`openapi.json`)
- **Serving:** swagger-ui (utoipa-swagger-ui) mounted in the API + the same spec embedded in the mdBook docs site (interactive reference)
- **mdBook integration:** docs/ book with: platform overview, spec index (AC traceability), API reference (generated), schema registry reference (SPEC-009 namespaces + types), stream/event reference (SPEC-004), governance (AGENTS/specs links)
- **Generation pipeline:** `cargo xtask` or build script regenerates docs from code + specs — docs never drift from handlers (spec-vs-code convergence check)
- **Schema registry in docs:** list registered namespaces/types in the book (from SPEC-009)
- **Traceability:** each endpoint maps to spec/ACs in the docs

**Dependencies:** SPEC-013 (API to document), SPEC-009 (registry content), future Bit console (docs for component library)

## SPEC-015 (planned): Event-Driven Ecosystem Integration (EIP)

**Requirement (captured 2026-08-07):** Extend platform capabilities with **Enterprise Integration Patterns** (messaging) for ecosystem integration:
- When a new **CRUD event** happens, **users are suggested possible actions**
- **Integrated applications receive event metadata** to take action (exec pipelines, update internal data, etc.)
- **AI agents autonomously execute actions** from events
- Reference: https://www.enterpriseintegrationpatterns.com/patterns/messaging/

**Roadmap fit (reuse-first):** the SPEC-004 commit stream IS the event backbone; EIP patterns map onto existing machinery:
| EIP pattern | Our mechanism |
|-------------|---------------|
| Event Message / Publish-Subscribe | SPEC-004 stream + typed DomainEvents |
| Message Channel | topic filters (entity / moderation / scope) |
| Message Router | subscription filters (content-based) |
| Channel Adapter / Envelope Wrapper | **webhook delivery + metadata envelope** (new) |
| Command Message | SPEC-011 pipeline commands / change requests |
| Event-Driven Consumer | agents/apps subscribing |
| Idempotent Receiver | SPEC-012 correlation ids (retry-safe delivery) |
| Message Expiration | batch TTL (SPEC-011) |

**Genuinely new surface:**
- **Subscription registry** (API/console): integrated apps register webhook endpoints + filters; events delivered as metadata envelopes with **suggested actions** per event type
- **Suggestion metadata**: event envelope carries possible next actions (e.g. PropertySetSaved → suggest Review/Promote/Approve) — feeds user suggestion UI and agent autonomy
- **Agent action channel**: AI agents subscribe and execute actions (change requests) — ride SPEC-011/SPEC-002 machinery
- Webhook delivery guarantees: at-least-once + idempotent receiver (corr ids), retries/backoff, webhook secrets (SPEC-014 auth), rate limiting

**Dependencies:** SPEC-004 (stream), SPEC-011 (pipelines), SPEC-012 (correlation), SPEC-013 (API surface for subscription registry), SPEC-014 (auth)

## SPEC-014 (planned): Authentication & Verified Principals

**Requirement (captured 2026-08-07, referenced by SPEC-013 R-18):** replace the v1 `X-Principal` header with verified principals — sessions/roles from SPEC-003, verified commit authors, webhook secrets. Mandatory before production (audit integrity). Detailed design deferred; noted here per relationship hygiene.

## SPEC-013 (planned): API Layer (axum REST)

**Requirement (captured 2026-08-07):** the missing HTTP surface — referenced as "future: axum application layer" in SPEC-006's crate structure. 8 specs are implemented with **no REST API** (only the SPEC-004 SSE stream). Unblocks: G6/Bit frontend, SPEC-011 pipeline batch API, SPEC-012 audit queries, SPEC-009 namespace console.

**v1 scope (DISCUSS):** core CRUD (entities, property sets, resolve), moderation (submit/decide/apply, promotion), audit queries, SSE stream mount, registry namespace listing + additionalType warn-mode; principals via simple auth header (SPEC-003 wiring documented); philosophy enforcement (schemaless warn-not-fail) applied at the boundary.

**Design questions:** framework (axum ✓), auth (v1 simple principal header; verified authors risk), error contract (JSON-LD-ish), validation warnings payload shape.

**Dependencies:** SPEC-001/002/003/004/006/009/012 (all implemented) — the API is a thin composition layer

**Requirement (captured 2026-08-07):** Auditability is a **platform-wide** concern, not pipeline-only: every mutation — manual edits, moderation decisions, promotions, pipeline batches — must be traceable (who/what/when/why) and queryable. Follow-up to the Graph Conductor roadmap fit (RFC #133), where the RFC's event-store auditability was compared to our commit log.

**What exists already:**
- Commit log (SPEC-006): every write is a TerminusDB commit with author/message + diff — the audit backbone
- Provenance records (SPEC-002) for promotions; time-travel via `resolve_at`
- Structured message tokens (`ps:`/`ent:`, SPEC-004)

**Gaps to close:**
1. **Moderation ledger is NOT persisted** (SPEC-002 in-memory) — who approved/rejected what, when, with which request is lost on restart (biggest gap)
2. **No correlation/causation IDs** — trace a unit of work (pipeline batch, UI session, API call) through the log
3. **No revert markers** — a reverting commit is indistinguishable from a normal one (`is_reverted`/`reverted_by` semantics)
4. **No audit query surface** — needs by entity / scope / actor / time range / event type; today only raw `log()`
5. **Principal-verified authors** — commit `author` must come from the authenticated SPEC-003 principal, not a caller-supplied string (spoofable today)

**Design sketch:**
- Audit = **projection over the commit stream** (SPEC-004 events + commit log + persisted ledger decisions) — no separate audit store; TerminusDB commits are append-only by nature
- Persist the ledger (change requests + decisions) via the repository (SPEC-006) → also unblocks the SPEC-002 activity stream
- Correlation IDs on all write entry points (pipeline/batch/API/session)
- Audit API: `GET /audit?entity=&actor=&scope=&from=&to=` (future API layer)

**Dependencies:** SPEC-002 (ledger persistence), SPEC-003 (principal-verified authors), SPEC-004 (event projection), SPEC-006 (commit backbone), SPEC-011 (pipeline correlation IDs)

## SPEC-011 (planned): Batch Ingestion Pipelines & Workflows

**Requirement (captured 2026-08-07):** Organizations and users build **pipelines/workflows** with different technologies — crawlers, Airflow, agents/MCP — to add **batches** of nodes, edges, combos, creative works, actions, etc. Comes with **race conditions, conflicts, and other design challenges/trade-offs**.

**Reference proposal (fetched 2026-08-07):** gustavorps' RFC "Graph Conductor" — github.com/reconurge/flowsint/issues/133 — a coordination layer for multi-agent graph mutations: command queue, region locking, version-vector OCC, event sourcing + agnostic revert, idempotency registry, role-scoped permissions, DLQ, anti-corruption layer. Stack: Redis/Neo4j/PG/Celery.

**Roadmap-fit brainstorm: done (docs/brainstorm/graph-conductor-roadmap-fit.md)** — conclusion: most mechanisms already exist in our stack (version guards = version vectors; commit log = event sourcing; SPEC-003 Policy = role permissions; TerminusDB branches = optimistic region isolation, replacing pessimistic locks). Genuinely new: batch API with idempotency keys, fast-track auto-approval for trusted pipelines, revert/undo surface (time-travel), DLQ, pipeline identity, MCP server integration (fork has mcp-server crate). **Pipeline audit requirements (correlation IDs, revert markers, batch traceability) live in SPEC-012 (platform-wide audit).**

**Design questions to resolve in the spec:**
- Batch write semantics on top of the moderation ladder (SPEC-002): batch = many change requests? one request per batch? fast-track auto-approve for trusted pipelines
- Idempotency: batch-level idempotency keys; retries must not duplicate entities/edges
- Conflicts: version-guard (exists) vs merge (TerminusDB merge verified) vs branch-per-pipeline (optimistic)
- Commit-per-write vs batched commits (SPEC-006 R-10)
- Pipeline identity + audit: pipeline = agent acting on behalf of org (SPEC-003; commit `author` = pipeline id)
- Revert: time-travel/reset via commit log (SPEC-006 resolve_at)
- Technology surface: HTTP batch API (future API layer), Airflow plugin, MCP server

**Dependencies:** SPEC-002 (moderation/fast-track), SPEC-003 (principals), SPEC-006 (commits/batching), SPEC-004 (streaming progress events), future API layer

**Requirement (captured 2026-08-06):** Nodes, edges, and combos can be **merged** — two or more graph primitives collapse into one entity.

**Key design questions to resolve in the spec:**
- Identity: merged entity takes which UUID? (new UUID + provenance, or survivor's UUID?)
- Content: how scoped property sets (SPEC-001) combine — union/conflict resolution per scope layer
- Moderation: does a merge require a change request (SPEC-002) and record provenance?
- Attachments: entity scope attachments (SPEC-003) union?
- History: prior versions/entities retained for audit?
- Relationship to soft-delete and promotion ladder
- Downstream impact: edges referencing merged nodes must be rewired

**Dependencies:** SPEC-001 (identity), SPEC-002 (moderation), SPEC-003 (attachments)

## SPEC-006 (planned): TerminusDB Git-Like Version Control & Temporal Architecture

**Requirement (captured 2026-08-06):** Leverage TerminusDB's Git-like version control — combining **JSON** (document) and **RDF** (graph) data models with high-precision **temporal reasoning** — to improve the platform's architecture and design patterns.

**Candidate architectural gains to evaluate in the spec:**
- **History (SPEC-002 R-3):** replace in-memory per-scope `history` vectors with TerminusDB commits/branches (time-travel, diff, rollback for free)
- **Promotion ladder (SPEC-002):** map promote/adopt onto branch → merge → commit semantics (brainstorm idea #3 "Git-for-Knowledge-Layers")
- **Provenance (SPEC-002):** commit metadata as the audit trail instead of separate `Provenance` records
- **Temporal reasoning:** version-as-of queries for `resolve(entity, scope, at: DateTime)` — time-travel resolution across scope layers
- **JSON+RDF dual model:** scoped property sets as JSON documents; edges/combo topology as RDF triples — schema decisions for SPEC-001 R-2 and cross-org sharing queries (SPEC-003)
- **Conflict detection:** TerminusDB merge conflict handling as the basis for competing-promotion resolution

**Gate:** ~~blocked on R-1~~ — **R-1 partially verified** (2026-08-06, github.com/terminusdb/terminusdb README): commits/diff/push-pull ✓, time-travel ✓, Allen-interval temporal reasoning ✓, JSON Git-for-Data ✓, WOQL/GraphQL/REST ✓, Rust client ✓; subscriptions ✗. Remaining unknowns: `@to` schema specifics, Rust client maturity, subscription availability — verify hands-on via local Docker server before spec creation.

**Dependencies:** SPEC-001 (R-2 schema pipeline), SPEC-002 (R-3 history, promotion), SPEC-003 (scope queries)

## SPEC-007 (planned): AI Code Quality Gauntlet

**Requirement (captured 2026-08-06):** Operationalize Robert C. Martin's ("Uncle Bob") approach to AI-generated code: instead of line-by-line human review, subject AI output to an automated "gauntlet" of strict static analysis, behavioral constraints, and verification gates.

**Candidate gates to define in the spec:**
- **Static quality & structure:**
  - Cyclomatic complexity ceilings (max conditional branching per function)
  - Dependency & coupling analysis (no circular deps, no architectural boundary violations)
  - Module/function limits (file length, function line count, argument count — SRP enforcement)
- **Testing & verification:**
  - Mutation score threshold (inject mutants; tests must catch them) — aligns with existing `.standards/mutation-testing.ai.yaml`
  - Code & branch coverage thresholds (line + conditional path coverage) — aligns with `.standards/full-coverage-testing.ai.yaml`
  - Executable behavioral acceptance tests (Gherkin/BDD scenarios run in CI) — aligns with `features/*.feature` + `.standards/behavior-driven-development.ai.yaml`
- **Philosophy shift:** humans set structural constraints + acceptance suites; AI code passes gates automatically — humans act as system architects, not line readers

**Tooling questions for the spec:** cargo-geiger / cargo-cyclomatic-complexity / rustfmt/clippy-as-gauntlet, cargo-mutants (mutation), tarpaulin/llvm-cov (coverage), cucumber-rs (Gherkin execution), CI wiring (UDS `test`/`lint`/`security` commands in `uds.project.yaml`).

**Dependencies:** none blocking (independent of R-1/R-2); requires CI wiring

## SPEC-008 (planned): terminusdb-rs Fork — JS-Client Feature Parity

**Requirement (captured 2026-08-06):** Implement the missing TerminusDB Rust-client features that the TypeScript/JavaScript client already has, in a **fork at https://github.com/gustavorps/terminusdb-rs** (fork of ParapluOU/terminusdb-rs), added as a **git submodule**.

**Features to port from the JS client (per ParapluOU repo README "Future Development"):**
- Branch management operations
- Push/pull/clone (remote database operations)
- Streaming operations
- Patch/diff operations
- Schema migration tools
- Advanced authentication methods

**Also blocking/supporting:**
- Unblocks SPEC-006 R-9 (branch-per-scope promotion mapping) and SPEC-004 (streaming)
- Reduces SPEC-006 R-7 (Rust client maturity risk) — forked dependency, in-repo

**Workflow:** add submodule (e.g. `third_party/terminusdb-rs`) → spec per feature (SDD) → TDD against the real server (Docker) or recorded fixtures

## SPEC-009 (planned): Multi-Namespace Versioned Schema Registry

**Brainstorm complete (docs/brainstorm/schema-registry-core-set.md, 2026-08-07) — decisions locked:**

**Core schema set (ships in knowledge-domain, protobuf):**
- 5 kinds via discriminated field + Thing-minimal properties: Node, Edge, Combo, CreativeWork, MediaObject, Action
- Edge = Thing + `subject`/`object`/`relationship`; Combo = Thing + members
- Everything IS a schema.org `Thing`; `additionalType` (registry-validated) is the extension seam
- schema.org `Action`/`AssessAction` tree records moderation **user intent** (ChooseAction→VoteAction, IgnoreAction, ReactAction→Like/Disagree/Endorse, ReviewAction); SPEC-002 ledger remains workflow state

**Scope decisions:**
- v1 = two namespaces only: `core` + `org.schema.v1` (curated pinned schema.org subset); multi-namespace console/tooling deferred
- **RISK (explicit): multi-schema-namespace complexity** — schema.org first
- Cross-domain mapping (OCSF, healthcare, FiBO) = long-term goal; schema.org is the seed/alignment standard

**Registry design elements:**
- Protobuf: `package` directive = namespace; immutable field tags (never changed/reused) = wire compat without runtime registry
- Stored in TerminusDB: FileDescriptorSets + derived schemas
- **Metadata property on entities: native domain datastructure + schema versioning pointer**
- **Registry UX: orgs define supported namespaces; selectable per workspace and project** (SPEC-003 scope instances)
- **Moderation action records: PostgreSQL; PG19 native SQL/PGQ for graph queries over AssessAction→moderation graphs** `[user-provided; verify PG19 SQL/PGQ support]`
- Tag budget 1–15 for core; tag-immutability lint; properties as resolvable IRIs
- Registry drives antd drawer form generation (property metadata → generic form renderer)

**Deferred:** derived-artifact cache, structural conformance validation, namespace explorer console surface (later)

**Dependencies:** SPEC-001 (schema model/codegen), SPEC-006 (TerminusDB storage), SPEC-003 (per-scope namespace selection), future API layer, Bit web console

## SPEC-010 (planned): AWS Remote Build & Test Offload

**Requirement (captured 2026-08-07):** Offload builds/tests that take **more than 5 seconds** to AWS, **copy the results back to the local machine** to save local time, and use **Pulumi as IaC** to **destroy the infrastructure after the workflow finishes** to save cost.

**Design elements to resolve in the spec:**
- **Trigger rule:** any build/test invocation whose estimated duration > 5s runs remotely (the fork's TerminusDB-from-source build is the prime candidate — minutes locally)
- **Instance choice:** ARM64 (Graviton) spot instances for Apple Silicon parity (same target triple `aarch64-apple-darwin`? — needs cross-check: Linux ARM ≠ macOS binaries! Local build output is macOS-specific → remote builds must produce *artifacts* (test results, build logs, cargo-check status), NOT macOS binaries — or use `cargo-build` in the remote for *verification* only and copy back only results/reports)
- **Round-trip:** S3 (or `aws s3 cp` / rclone) for transfer: source tarball up, results/artifacts down; local machine keeps final outputs
- **IaC:** Pulumi program (TypeScript or Go): EC2 spot + security group + S3 bucket; `pulumi destroy` in a guaranteed teardown path (workflow finally-block / CI job cleanup step)
- **Cost control:** spot instances, t2/t3g sizing, idle-timeout auto-termination, destroy-on-completion as a hard gate
- **Cache:** remote sccache/cargo target reuse across runs (EBS snapshot or S3 cache) — otherwise cold-start cost dominates
- **Security:** no secrets in workflow images; IAM-scoped creds; destroy even on failure

**Open questions:** build verification vs artifact production on Linux (macOS targets), network transfer vs local build time break-even (~5s threshold implies very cheap fast-path), running remote tests against TerminusDB servers (Docker-in-remote)

**Dependencies:** SPEC-007 (gauntlet gates can trigger remote runs), fork build times (current bottleneck)
