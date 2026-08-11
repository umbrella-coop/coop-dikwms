# Spec Backlog

Planned specs not yet created. Add requirements captured during development here so they are not lost.

> **Intake rule (added 2026-08-07):** every spec source feeds this queue — brainstorm outcomes, user requests, dependency findings, and spec-relationship references. A spec is created from the queue, not on demand.

## SPEC-004 (planned): Live Streaming via the Commit Stream

**Requirement (captured 2026-08-06):** live stream data for nodes and combos — the original problem statement. **Source:** brainstorm idea #5 (CDC + cursor, Agg 4.1) — referenced as "planned" in SPEC-001/002/006/008 relationships but never created (gap).

**Status:** **fully de-risked by SPEC-008** — SSE plugin endpoint is dead on v12 (404); the **native commit stream is verified working** (commit advances + `commit_added_entities_ids` diff).

**Design sketch:**
- Live updates = commit-log cursor over `terminusdb-repository` (poll/`log_iter` diff per cursor position)
- Domain events: node created, property promoted, combo regrouped, change-request lifecycle (feeds SPEC-002 activity stream)
- Client reconciliation via cursor (no redraw storms — G6 consumes domain events, not raw deltas)
- PostgreSQL events for moderation actions (PG19 SQL/PGQ — verify) vs TerminusDB commit stream for knowledge changes — boundary decision for the spec

## SPEC-017 (planned): BDD Drift Handling & DDD Bounded-Context Organization

**Requirement (captured 2026-08-07):** Handle **BDD drift** and organize Gherkin features/tests by **DDD bounded context**.

**Design elements to resolve in the spec:**
- **BDD drift:** `.feature` files can drift from specs (ACs) and implementation — detection + discipline:
  - Cross-artifact checks (`/sdd analyze` style): orphan scenarios (no AC), uncovered ACs, stale scenario language
  - Regeneration/derivation rule: features derive from spec ACs (`/derive bdd`); changes flow spec → feature → step definitions → implementation (forward-derivation single spine)
  - Feature files tagged `@SPEC-NNN @AC-N` (already the convention) + a drift check runnable in CI (SPEC-007 gauntlet)
- **DDD bounded-context organization:** the flat `features/SPEC-*.feature` + per-crate tests organized by bounded context mirroring `backend/crates/*`:
  - `features/knowledge-domain/`, `features/moderation/`, `features/access/`, `features/registry/`, `features/stream/`, `features/api/`
  - Each bounded context owns its feature files, step definitions, and test slices; specs stay global (SPEC-NNN) with a Context mapping
  - Cross-context scenarios (promotion → audit) explicitly marked or placed in the owning context with references
- **Context map:** docs section mapping bounded contexts → crates → spec slices → feature dirs (AI-navigability)

**Dependencies:** SPEC-007 (drift checks in CI), existing BDD derivations (features/SPEC-*.feature), crate layout

## SPEC-021 (planned): Frontend E2E Suite (Playwright) — held

**Requirement (captured 2026-08-07, source: orchestrator):** hold SPEC-018's deferred E2E features; until then, frontend testing is **manual, orchestrator-driven via the UI** (`bit run app -p <port>` + browser).

**Held items (from SPEC-018, deferred):**
- AC-3 E2E assertion: `window.__APP_READY__` wait + canvas render + drawer open + live-update lands
- AC-6 E2E: SSE genesis replay + cursor reconnect in a real browser
- Drawer save-failure error UI (antd form submission — unreliable under jsdom)
- Playwright Page Object Models over the data-testid contract (SPEC-018 AC-1 selectors)
- Integration with SPEC-017 (bounded-context feature dirs + drift checks)

**Dependencies:** SPEC-018 (conventions in place), SPEC-017 (E2E org), SPEC-010 (remote test exec, later)

## SPEC-020 (planned): Context Reorganization — data-graph rename & namespace moves

> **PROMOTED (2026-08-08):** frontend portion created as docs/specs/SPEC-020-dikwms-frontend-reorg.md — entry kept for provenance. Backend portion (`knowledge-domain` → `data-graph` crate rename) remains deferred here.

**Requirement (captured 2026-08-07, source: orchestrator DDD review):**
- Rename backend crate `knowledge-domain` → **`data-graph`** (bounded context: Data Graph)
- Frontend moves (via `bit move`, never re-create):
  - `ui/graph` → `ui/data-graph/canvas`
  - `ui/select-layout` (extracted from graph) → `ui/data-graph/select-layout`
  - `hooks/use-event-stream` → `ui/data-graph/hook-use-event-stream` (hooks live in their owning context)
  - `apps/coop-graph-app` → `app`
- Reserved namespaces (created when first component lands): `data-graph-governance/`, `iam/`, `data-schema-registry/`
- `frontend/check-layout.mjs` updated to the context map

**Execution policy (orchestrator, 2026-08-07):** moves and renames are **opportunistic** — applied as components are touched, never a dedicated batch restructure.

**Dependencies:** DDD context map v2 (orchestrator-approved), SPEC-019 (scope), SPEC-018 (conventions)

## SPEC-019 (planned): Naming & Bit Scope Correction — coop-codes.network-graph

**Requirement (captured 2026-08-07, source: user request):** replace the placeholder `grps.coop-graph` Bit scope with the real organization scope **`coop-codes.network-graph`** (Bit dev org `@coop-codes`), and correct the "graph-network" naming drift to `network-graph` across the project.

**Scope of the fix:**
- `frontend/workspace.jsonc`: `defaultScope` + workspace-config key → `coop-codes.network-graph` (done in the spike)
- Component namespaces: `frontend/coop-graph/` → `frontend/network-graph/` (Bit `bit rename` / dir restructure)
- References in docs/AGENTS/specs that use `grps.coop-graph` or "graph-network" for the Bit scope
- Repo directory name `coop-graph-network` stays (it is the repo), only Bit-scope naming is corrected
- Verify: `bit status` clean, component ids resolve to `@coop-codes/network-graph.*`

**Dependencies:** frontend spike (Bit workspace), SPEC-018 (frontend platform)

## SPEC-018 (planned): Frontend Platform — AI-Agent-Friendly E2E Testability

> **PROMOTED (2026-08-07):** created as docs/specs/SPEC-018-frontend-e2e-testability.md — entry kept for provenance.

**Requirement (captured 2026-08-07, source: user request + .wip/todo-friently-e2e-to-code-agents.md):** the frontend (G6 + antd + Bit microfrontends) SHALL be built AI-agent/E2E-friendly per the conventions in `.wip/todo-friently-e2e-to-code-agents.md`:
- Standardized `data-testid` on interactive components (never volatile class names/UI copy)
- DOM state reflection: `data-loading`, `data-state`, `data-error` attributes for async/lifecycle states
- Semantic HTML + explicit ARIA (`role="dialog"`, `aria-expanded`, `aria-label`)
- Machine-readable hooks: `window.__APP_READY__` / `__REACT_HYDRATED__` readiness flags in dev/test builds
- Structured JSON error output from error boundaries + network failures
- Frontend `AGENTS.md`: E2E framework conventions (Playwright), selector scheme, test scripts; Page Object Models as reusable abstractions

**Status:** the frontend spike (G6 graph + antd registry-driven drawer + SSE live updates, Bit workspace) implements these conventions first; this spec formalizes them when the spike stabilizes.

**Dependencies:** SPEC-013 (API), SPEC-016 (OpenAPI contract), SPEC-009 (registry-driven forms), SPEC-004 (live stream), SPEC-017 (BDD/E2E org)

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

## SPEC-012 (planned): Audit & Traceability (platform-wide)

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

## SPEC-005 (planned): Graph Primitive Merge

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

**Requirement (captured 2026-08-06):** Implement the missing TerminusDB Rust-client features that the TypeScript/JavaScript client already has, in a **fork at https://github.com/gustavorps/terminusdb-client-rs** (fork of ParapluOU/terminusdb-rs), added as a **git submodule**.

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

**Workflow:** add submodule (e.g. `third-party/terminusdb-client-rs`) → spec per feature (SDD) → TDD against the real server (Docker) or recorded fixtures

## SPEC-009 (planned): Multi-Namespace Versioned Schema Registry

**Requirement (captured 2026-08-06):** a multi-namespace, versioned schema registry managed by knowledge-domain, stored in TerminusDB; core schemas only in code; schema.org as the first namespace; namespaces added via API/web console using Protocol Buffers (package-directive namespaces, immutable field tags).

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

## SPEC-022 (planned): Bit DX workflow — HMR, linking, cache hygiene

**Requirement (captured 2026-08-08, source: user request — bit.dev blog "Local Cross-Project Component Development with Bit Link Target"):**
- Vite HMR for the org scope in the app dev server (`server.watch.ignored` + `optimizeDeps.exclude: ['@coop-codes']`)
- `bit link --target <path> --peers` documented for future cross-repo consumers (symlink live components + peers)
- Cache/re-link hygiene: clear `node_modules/.vite` after renames/moves; re-establish links after installs; `bit watch` for continuous compile
- Frontend npm scripts: `dev`, `watch`, `test`, `check`, `fix-links`, `build`

**Verdict (roadmap fit):** reuse — thin surface (config + scripts + docs), no product behavior. Fold into frontend AGENTS.md.

**Dependencies:** SPEC-020 (workspace layout), SPEC-018 (frontend conventions)

## SPEC-023 (planned): Upstream Contribution Ladder — terminusdb-rs fork → ParaplouOU

**Requirement (captured 2026-08-10, source: user request + brainstorm):** contribute the fork's verified v12/auth work back to upstream `ParaplouOU/terminusdb-rs` as small PRs, each branched from upstream `main`.

**Brainstorm complete (2026-08-10) — decisions locked (rebuttal-accepted):**
- **Trust ladder, three sequential PRs, never combined:**
  - PR-1 = `Authorization-Remote` header-casing fix (one-liner, extracted from `3f04b24`)
  - PR-2 = bearer/api-key auth (`3bd396d`) — the documented JS-client parity gap (SPEC-008)
  - PR-3 = remaining v12 collaboration fixes + pruned v12-verification tests (`3f04b24` remainder + subset of `11d525b`)
- **Excluded from any PR:** `580b840` (rustfmt sweep, 146 files), `13cb418` (local build tooling), fork-specific Rebase/Apply merge semantics
- **Packaging discipline:** 10-min upstream duplicate check → branch from upstream `main` → cherry-pick → verify on upstream nightly + clippy → open PR with a 3-line v12 evidence note (changeset-sse 404, header casing, v12 push/pull paths)
- No issue-first waiting — dup-check only

**Dependencies:** SPEC-008 (archived — 7/7 AC verification evidence vs real v12.1), fork `dev` branch work (auth/collab/tests), RISK-004 (fork pin — upstream landing relaxes it)

## SPEC-024 (planned): Multi-Version TerminusDB CI Matrix (docker + act)

**Requirement (captured 2026-08-10, source: user request):** add a docker-based integration-test matrix to the fork's GitHub workflows covering `terminusdb/terminusdb-server` v12.x (latest tag = priority) and v11.x; runnable locally via `act` (act 0.2.89 installed); failures on non-latest versions are **report-only** — never fixed in code; generate a report artifact to open an issue manually.

**Design elements to resolve in the spec:**
- Matrix axes: `latest` (blocking, priority) + v12.x pin + v11.x pins (report-only, non-blocking)
- act compatibility: container-based services; document `act -j <matrix-job>` local invocation
- Report: markdown report artifact + job summary; NO auto-issue (manual `gh issue create`)
- Runs **alongside** the existing embedded-server job (`tests.yml`, v12.1-rc-paraplu.1) — not replacing
- Client is v12-only (v11 dropped during v12 upgrade) → v11 failures expected; that is the point of report-only

**Dependencies:** fork `.github/workflows/tests.yml`, SPEC-023 (evidence + issue material), act local tooling

## SPEC-025 (planned): Developer Experience — fast builds/tests + CONTRIBUTING.md

**Requirement (captured 2026-08-10, source: user request):** improve DX for contributors by speeding up local build and test across environments (Linux, macOS, Windows). **Constraint (user):** must NOT impose `.cargo/config.toml` or dependency installs on new contributors — provide **guidelines (CONTRIBUTING.md)** instead for a convenient/fast dev environment.

**Raw material (fork commit `13cb418` — excluded from PR-1..3 as dev-tooling, now the PR-4 basis):**
- Cargo.toml dev-profile tweaks: `codegen-units = 256`, `opt-level = 0`, `debug = 1`, `[profile.dev.package."*"] opt-level = 2` — compile-time-only, safe to ship (no linker/toolchain imposition)
- `.cargo/config.toml` contents (lld, `-Z share-generics`, `RUST_TEST_THREADS=1`, sccache wrapper) — **excluded from the PR** (imposes linker/nightly on contributors); move into CONTRIBUTING.md as optional guidance (env vars: `RUSTFLAGS`, `CARGO_PROFILE_DEV_*`)
- `.mise.toml` — excluded; document as optional tooling

**Upstream facts:** no CONTRIBUTING.md exists; workspace has only `[profile.release]` (lto, codegen-units=1 — slow release builds); per-OS deps (clang/libclang/protoc/libgmp/openssl; SWI-Prolog for embedded server — Windows-first-contribution guidance needed: client/schema crates work, embedded-server tests are Linux/macOS-first or WSL).

**Dependencies:** SPEC-023 (ladder), SPEC-024 (CI matrix — CI speed is a separate axis), fork `13cb418`
