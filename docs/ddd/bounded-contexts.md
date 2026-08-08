# DDD Bounded Contexts — Canonical Reference (v2)

> **Status:** Active · **Version:** 2 (orchestrator-approved 2026-08-07)
> **Provenance:** moved from `docs/brainstorm/ddd-bounded-contexts.md` (brainstorm report, v1) → canonicalized to `docs/ddd/` 2026-08-08. History preserved in git.
> **Governed by:** SPEC-020 (context reorg — opportunistic moves) · ADR-001 (context map) · ADR-002 (frontend layout) · ADR-003 (testid naming) · UDS options `ddd-bounded-context-org` + `frontend-component-conventions`

This is the authoritative context map for the **dikwms** codebase. It guides:
- where new code lands (frontend + backend),
- how existing code is moved/renamed (SPEC-020, opportunistic),
- how the backend refactor (crate `knowledge-domain` → `data-graph`) proceeds.

## 1. Context Map (v2)

| Bounded context | Backend container | Frontend namespace | Status | Responsibilities |
|---|---|---|---|---|
| **Data Graph** | `data-graph` (rename from `knowledge-domain` — pending, SPEC-020) | `ui/data-graph/` (node-drawer) + `ui/data-graph-antv-g6/` (canvas, layout-select — G6 rendering engine) + `hook/use-data-graph-sse` | **active** | entities, scoped property sets, resolve (nearest-wins), G6 rendering, live-event consumption (SSE via `stream-api`) |
| **Governance** | moderation + audit logic (co-located in `data-graph` + `terminusdb-repository` today) | `ui/data-graph-governance/` | reserved | change requests, decisions, promotion ladder, moderation ledger, audit trail, correlation, revert |
| **IAM** | Policy ACL (co-located in `data-graph` today) | `ui/iam/` | reserved | scope instances, principals, ACL enforcement |
| **Data Schema Registry** | `schema-registry` (standalone ✓) | `ui/data-schema-registry/` | reserved | namespaces, additionalType, versioned schema docs |
| **App Shell** | `api` (transport only — no business logic) | `app` (was `apps/coop-graph-app`, SPEC-020) | **active** | composes contexts, REST surface, SSE fan-out |

### Dissolved: Stream
`stream-api` and `hook-use-event-stream` are **not a context** — they are transport/infrastructure *of the Data Graph context* (SPEC-004 AC-4 live events). Hooks live at `hook/<name>` with the context in the hook name: `hook/use-data-graph-sse`. The `stream-api` crate stays until volume justifies merging (deferred).

### Reserved namespaces
`ui/data-graph-governance/`, `ui/iam/`, `ui/data-schema-registry/` (and backend `data-graph-governance`, `iam` crates) are **created when their first component lands** — no empty directories now.

### DIKW mapping (product vision, AGENTS.md)
| Layer | Contexts |
|---|---|
| Data | Data Graph (raw graph primitives) |
| Information | Data Schema Registry (curation) · IAM (scoping) |
| Knowledge | Governance (promotion ladder, moderated truth) |
| Wisdom | audit/analytics (Governance backend) |

## 2. Ubiquitous Language

| Term | Meaning |
|---|---|
| entity | a graph node identity (uuid) |
| property set | values keyed by (entity, scope-instance) |
| scope instance | org/workspace/project; `COMMON_INSTANCE` = global root |
| resolve | nearest-wins walk up the scope-instance ancestor chain |
| change request | proposed edit awaiting governance decision |
| promotion | ladder step making a change request into governed truth |
| namespace | registry-scoped schema (e.g. `core.v1`, `org.schema.v1`) |
| additionalType | data may declare a namespace type; mismatch = warning, never failure |
| commit / cursor | provenance anchors for the live stream |

## 3. Frontend Organization (focus)

### 3.1 Current layout (realized 2026-08-08, SPEC-020 implemented)

```
frontend/dikwms/                        (Bit scope: coop-codes.dikwms)
├── app/                                (app shell: app)
├── type/
│   └── core-v1/                   (DataGraphNode/Edge/Combo — DIKW Data primitives; mirrors registry `core.v1`)
└── ui/
    ├── data-graph/
    │   └── node-drawer/                (← ui/entity-drawer, testids node-*)
    ├── data-graph-antv-g6/             (G6 rendering engine)
    │   ├── canvas/                     (← ui/graph)
    │   └── layout-select/              (extracted from canvas; testid layout-select)
    ├── data-graph-governance/          (reserved — created on first component)
    ├── iam/                            (reserved)
    └── data-schema-registry/           (reserved)
```

Hooks live at `frontend/dikwms/hook/<name>` with the context in the hook name
(`hook/use-data-graph-sse` — SSE transport for the Data Graph context).
The layout above is the **reality**; the spec and this document follow the
codebase, never the inverse (Working software over comprehensive
documentation).

### 3.2 Rules

1. **Type-first namespace, context in path or name:** `ui/<context>/<name>` for visual components, `hook/<name>` for hooks (context encoded in the hook name, e.g. `use-data-graph-sse`), `type/<name>` for shared wire-agnostic types (`type/core-v1`), `app/<name>` for shells.
2. **Bit moves only, never re-create:** `bit move`/`bit rename -s/-p` preserve component ids, dependencies, and test specs.
3. **testids are a stable contract (ADR-003):** kebab-case, intent-descriptive (`layout-select`, `node-drawer`, `node-save-btn`) — testids do **not** change on pure moves; the E2E POM layer (SPEC-021) depends on them.
4. **Dependency rule:** components communicate via interfaces; the API is the translation layer. No cross-context imports (documented rule; CI import-boundary check deferred). Frontend context X may only call API routes of context X (+ shared `app/dikwms`).
5. **Reserved namespaces** are created on first landing component, never pre-provisioned.
6. **check-layout.mjs (v2, SPEC-020 AC-8):** enforces `{app, hook, type, ui}` + context whitelist (`data-graph`, `data-graph-antv-g6`, `data-graph-governance`, `iam`, `organization`, `project`, `workspace`, `data-schema-registry`) + `index.ts` presence.

### 3.3 Migration history (completed 2026-08-08)

| From | To | Commit |
|---|---|---|
| `apps/coop-graph-app` | `app` | `1fc67fb` (scope+move batch) |
| `hooks/use-event-stream` | `hook/use-data-graph-sse` | `1fc67fb` |
| `ui/graph` | `ui/data-graph/canvas` | `1fc67fb` |
| `ui/entity-drawer` | `ui/data-graph/node-drawer` (testids `node-*`) | `1fc67fb`, `c0d3511` |
| (inside `canvas`) | `ui/data-graph/layout-select` | `f5d92c8` |
| `ui/data-graph/canvas`, `ui/data-graph/layout-select` | `ui/data-graph-antv-g6/{canvas,layout-select}` (G6 rendering engine namespace) | `(orchestrator, 2026-08-08)` |
| scope `coop-codes.network-graph` | `coop-codes.dikwms` | `1fc67fb` |

## 4. Backend Refactor Guide (toward v2)

### 4.1 Target state

```
backend/crates/
├── data-graph/            (← knowledge-domain — rename)
├── schema-registry/       (unchanged ✓)
├── stream-api/            (unchanged — Data Graph transport)
├── terminusdb-repository/ (unchanged — shared persistence infra, NOT a context)
└── api/                   (unchanged — App Shell transport)
```

### 4.2 Module → context mapping (current code)

| Current module | Owning context | Action |
|---|---|---|
| `knowledge-domain` entity/scope/resolve | Data Graph | **rename** crate → `data-graph` |
| `knowledge-domain` moderation (change requests, promotion ladder) | Governance | co-located today; extract to `data-graph-governance` crate when volume justifies (reserved) |
| `knowledge-domain` Policy ACL | IAM | co-located today; extract to `iam` crate when volume justifies (reserved) |
| `terminusdb-repository` audit ledger, ps-token property sets, `resolve_at` | Governance rules / shared infra | stays (infra implements context persistence) |
| `schema-registry` | Data Schema Registry | stays ✓ |
| `stream-api` SSE | Data Graph transport | stays |
| `api` routes/auth | App Shell | stays ✓ |

### 4.3 Rename mechanics: `knowledge-domain` → `data-graph`

Touch list (exact, verified):

- `backend/Cargo.toml` — workspace `members`: `crates/knowledge-domain` → `crates/data-graph`
- `backend/crates/knowledge-domain/` → `backend/crates/data-graph/`; in its `Cargo.toml`: `name = "data-graph"`
- Dependency refs in `api/Cargo.toml`, `stream-api/Cargo.toml`, `terminusdb-repository/Cargo.toml` (path + package name)
- Rust imports `use knowledge_domain::…` → `use data_graph::…` in:
  - `backend/crates/api/src/routes.rs`
  - `backend/crates/terminusdb-repository/src/lib.rs`, `src/audit.rs`
  - tests: `api/tests/*` (via routes), `stream-api/tests/spec_004_sse.rs`, `terminusdb-repository/tests/{spec_001,spec_004,spec_006,spec_012}*.rs`
- `knowledge-domain/tests/{spec_001,spec_002,spec_003}.rs` move with the crate (they are its tests)
- Docs referencing the crate name as *current layout* (AGENTS.md, this doc's §1) — spec references in **archived** specs (SPEC-001/002/003) are historical and stay untouched

Verification: `RUSTUP_TOOLCHAIN=nightly cargo check -p data-graph --tests` then full crate test pass; grep audit for zero residual `knowledge_domain`.

### 4.4 Dependency direction (invariant)

- `api` → every context crate (allowed — App Shell composes).
- A context crate may depend on `terminusdb-repository` (persistence) but **never on another context crate**.
- `terminusdb-repository` depends on domain types (`data-graph`) — infra implements context persistence; this is not a context-to-context edge.

## 5. Guardrails & Anti-Corruption

- **Schemaless-by-default:** registry validation warns, never rejects (SPEC-009 R-14) — registry context must never block data writes.
- **API = translation layer:** business rules live in contexts; `api` only routes/translates.
- **testids stable:** UI moves never change testid strings (POM contract, ADR-003).
- **Opportunistic only:** SPEC-020 moves ride along with feature work; no dedicated restructure batches.
- **Import-boundary CI check:** deferred (documented rule only).

## 6. Tracking & Open Items

- **SPEC-020 (backlog):** owns the moves above; update `check-layout.mjs` to v2 in the same change as the first move.
- **ADR-001/002/003:** active decisions; ADR source paths now point here.
- **Deferred:** CI import checks · generator customization (revisit at 10+ components) · `stream-api` merge into `data-graph` · Governance/IAM backend extraction · SPEC-021 E2E (uses the testid contract).

## 7. Change History

| Date | Change |
|---|---|
| 2026-08-07 | v1 brainstorm approved (BQS v1); ADRs + UDS options codified |
| 2026-08-07 | v2 orchestrator review: 5-context map, stream dissolved, DIKW vision, reserved namespaces |
| 2026-08-08 | Moved to `docs/ddd/bounded-contexts.md`; hardened with migration tables, rename mechanics, dependency invariants |
| 2026-08-08 | SPEC-020 frontend implemented: scope `coop-codes.dikwms`, layout realized (codebase feeds this doc) |
