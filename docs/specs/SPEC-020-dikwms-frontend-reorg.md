# Feature: SPEC-020-dikwms-frontend-reorg — dikwms Frontend Context Reorganization (Iterative)

<!-- status: Implemented -->
<!-- approved-date: 2026-08-08 -->
<!-- approved-by: orchestrator -->
<!-- created: 2026-08-08 -->
<!-- source: backlog SPEC-020 (promoted, narrowed to frontend; orchestrator directive 2026-08-08) -->

> **Living document:** this spec is updated via `## MODIFIED` deltas as each move lands. Status progresses Draft → Approved → Implemented → Archived as the iterative reorg completes. All sections reflect the current truth of the reorg.

## Overview

Iterative process to create and refactor sets of frontend components onto the **`dikwms`** Bit scope and a context-first layout — rename the scope, move the existing components, extract the layout selector, and keep the DDD documentation *reflecting the codebase* (Working software over comprehensive documentation). The number of future components is unknown; this spec defines the process + the initial concrete set.

## Requirements

- REQ-001: Rename Bit scope `coop-codes.network-graph` → `coop-codes.dikwms` (dir `frontend/network-graph/` → `frontend/dikwms/`)
- REQ-002: Move `frontend/network-graph/apps/coop-graph-app` → `frontend/dikwms/app` (bit move, never re-create)
- REQ-003: Move `frontend/network-graph/hooks/use-event-stream` → `frontend/dikwms/hook/use-data-graph-sse` (SSE = Server-Side Events)
- REQ-004: Move `frontend/network-graph/ui/graph` → `frontend/dikwms/ui/data-graph/canvas`
- REQ-005: Move + rename `frontend/network-graph/ui/entity-drawer` → `frontend/dikwms/ui/data-graph/node-drawer`
- REQ-006: New component `frontend/dikwms/ui/data-graph/layout-select` — decouple the layout selector from canvas; keeps `data-testid="layout-select"`
- REQ-007: **Codebase-first docs:** `docs/ddd/bounded-contexts.md` (+ AGENTS.md, ADR-002 as needed) reflects the actual codebase layout, updated in the same commits as the moves — the doc follows the code, never the inverse (Agile Manifesto: working software over comprehensive documentation)
- REQ-008: Iterative process: moves are opportunistic (applied as components are touched); `frontend/scripts/check-layout.mjs` updated to the new shape; spec kept in sync via deltas

## Technical Design

### Target layout (end state)

```
frontend/dikwms/                      (Bit scope: coop-codes.dikwms)
├── app/                              (app shells; ← apps/coop-graph-app)
├── hook/                             (hooks; context encoded in name)
│   └── use-data-graph-sse/           (← hooks/use-event-stream)
└── ui/
    ├── data-graph/
    │   ├── canvas/                   (← ui/graph)
    │   ├── node-drawer/              (← ui/entity-drawer)
    │   └── layout-select/            (NEW — extracted from canvas)
    ├── data-graph-governance/        (reserved — created on first component)
    ├── iam/                          (reserved)
    └── data-schema-registry/         (reserved)
```

### Scope rename mechanics

- `frontend/workspace.jsonc`: `defaultScope` → `coop-codes.dikwms`; `defaultDirectory` stays `{scope}/{name}`
- Bit: `bit rename --scope` (or per-component `bit move` onto new scope) — component ids become `coop-codes.dikwms/...`
- Dir: `frontend/network-graph/` → `frontend/dikwms/`
- Verify: `bit status` clean; ids resolve to `@coop-codes/dikwms.*`

### Move mechanics (each move)

- `bit move <old-id> <new-path>` — preserves component id dependencies, specs, and test specs; never re-create components
- Name changes ride along where the move IS the rename (e.g. `entity-drawer` → `node-drawer`): rename component function/export + dir in one move
- testid changes ride along ONLY where the component is renamed (node-drawer) or extracted (layout-select); otherwise testids are stable (ADR-003)

### layout-select extraction

- Extract the layout `<Select>` currently at `graph.tsx` (lines ~100–115, `data-testid="layout-select"`, `aria-label="Graph layout type"`) into `ui/data-graph/layout-select`
- Props: `value`, `onChange`, layout options list (9 options today); canvas consumes it
- testid stays `layout-select`; add `layout-select.spec.tsx` covering render + change callback

### check-layout.mjs v2

- Allowed top-level namespaces under `frontend/dikwms/`: `{app, hook, ui}`
- `ui/` entries must be context dirs from whitelist: `data-graph`, `data-graph-governance`, `iam`, `data-schema-registry`
- Each component dir must contain `index.ts(x)` (existing rule kept)
- Landed in the same commit as the first move (never before — must not fail on the current tree)

## Acceptance Criteria

- AC-1: Given the Bit workspace, When the scope is renamed to `coop-codes.dikwms`, Then `bit status` is clean and component ids resolve to `@coop-codes/dikwms.*` (per-component via `bit rename --scope`/`-s`)
- AC-2: Given the app shell, When it is moved to `frontend/dikwms/app`, Then `bit run -p 3100` serves it and the root/`@vite/client`/app modules return 200 (dev port per SPEC-018 R-28)
- AC-3: Given the SSE hook, When moved to `frontend/dikwms/hook/use-data-graph-sse`, Then its spec passes and the smoke canvas still receives live events
- AC-4: Given the canvas, When moved to `frontend/dikwms/ui/data-graph/canvas`, Then the graph renders with the layout selector working and its spec passes
- AC-5: Given the entity drawer, When moved and renamed to `frontend/dikwms/ui/data-graph/node-drawer`, Then the drawer opens from canvas selection, its spec passes, and its testids are renamed `entity-*` → `node-*` (`entity-drawer`→`node-drawer`, `entity-form`→`node-form`, `entity-form-error`→`node-form-error`, `entity-name-input`→`node-name-input`, `entity-save-btn`→`node-save-btn`)
- AC-6: Given the layout selector, When extracted to `frontend/dikwms/ui/data-graph/layout-select` and consumed by canvas, Then `data-testid="layout-select"` remains in the DOM and layout switching still works
- AC-7: Given the reorganized codebase, When docs are checked, Then `docs/ddd/bounded-contexts.md` and `frontend/AGENTS.md` describe the actual layout (scope `dikwms`, `app/`, `hook/`, `ui/<context>/`) — committed in the same commits as the moves
- AC-8: Given the new layout, When `node frontend/scripts/check-layout.mjs` runs, Then it validates `{app, hook, ui}` + context whitelist and reports green
- AC-9: Given each move, When it lands, Then the spec records a `## MODIFIED` delta with the commit hash and AC status — the spec is a living document

## Assumptions & Open Questions

### Assumptions

| # | Assumption | Impact Scope | Verification Method | Status |
|---|-----------|--------------|---------------------|--------|
| A1 | `canva` in the directive was a typo for `canvas` | REQ-004 | confirmed with orchestrator | Verified |
| A2 | New scope id is `coop-codes.dikwms` (org prefix stays) | REQ-001 | confirmed with orchestrator | Verified |
| A3 | Hooks live at `hook/` (singular) with context in the name | REQ-003 | confirmed with orchestrator | Verified |
| A4 | Backend `knowledge-domain` → `data-graph` rename stays out of scope (deferred in backlog) | REQ-008 | confirmed with orchestrator | Verified |
| A5 | testid renames (`entity-*` → `node-*`, five testids verified in code) are cheap now, before SPEC-021 E2E exists | REQ-005 | E2E not implemented | Verified |
| A6 | Component ids follow Bit convention `coop-codes.dikwms/<path>/<name>` | all | bit status clean check | Unverified → AC-1 |

### Open Questions

None — all resolved with the orchestrator (2026-08-08).

## Test Plan

- [ ] `bit test` — full frontend suite green after EVERY move (7 tests today + new layout-select spec)
- [ ] New: `layout-select.spec.tsx` (render + onChange callback)
- [ ] `node frontend/scripts/check-layout.mjs` green (v2, after first move)
- [ ] `bit status` clean; `bit list` shows `coop-codes.dikwms/*` ids
- [ ] Smoke: `bit run` app on dev port — root + modules 200, canvas renders, layout switch works, SSE events flow

## Rollout Plan

Iterative + opportunistic (SPEC-020 policy): each move rides on the next touch of its component — no dedicated batch restructure. Each landed move: code+test+doc delta in one commit, spec delta recorded (AC-9). Order suggested: scope rename first (unblocks id stability), then app → hook → canvas/node-drawer/layout-select as touched.


---

## MODIFIED — Implementation record (living document, 2026-08-08)

| AC | Evidence | Commit |
|----|----------|--------|
| AC-1 | scope `coop-codes.dikwms`; `bit status` clean (5 components ok); ids `coop-codes.dikwms.*` | `1fc67fb` |
| AC-2 | `bit run app -p 3100` → root + `@vite/client` 200 (smoke) | `1fc67fb`, `55a9280` |
| AC-3 | hook at `hook/use-data-graph-sse`; spec green (2 tests); canvas consumes it | `1fc67fb` |
| AC-4 | canvas at `ui/data-graph/canvas`; layout selector working; spec green | `1fc67fb` |
| AC-5 | node-drawer at `ui/data-graph/node-drawer`; testids `node-drawer/node-form/node-form-error/node-name-input/node-save-btn`; spec green | `1fc67fb`, `c0d3511` |
| AC-6 | `layout-select` component extracted; `data-testid="layout-select"` in DOM (canvas spec + own spec assert it); layout switch works | `f5d92c8` |
| AC-7 | `docs/ddd/bounded-contexts.md` + `frontend/AGENTS.md` + ADR-002 reflect realized layout, same commits as moves | `55a9280` |
| AC-8 | `node frontend/scripts/check-layout.mjs` green — `dikwms/{app,hook,ui}` + context whitelist (moved to `frontend/scripts/` per orchestrator) | `55a9280` |
| AC-9 | this delta; each move committed with AC refs | `1fc67fb`, `c0d3511`, `f5d92c8`, `55a9280` |

**Deviations:** none. Final suite: 9 tests green (4 files); `bit status` clean (5 components).

- **Spelling correction (2026-08-08):** canonical name is **`dikwms`** (DIKW+ms) — scope `coop-codes.dikwms`, dir `frontend/dikwms/` (the `diwkms` form used in early directives was a typo; corrected everywhere in the 2026-08-08 correction batch).

- **Post-implementation move (2026-08-08, orchestrator):** `canvas` + `layout-select`
  moved to new rendering-engine namespace `ui/data-graph-antv-g6/` (ids
  `coop-codes.dikwms.ui.data-graph-antv-g6.*`); `check-layout.mjs` whitelist +
  `docs/ddd/bounded-contexts.md` + ADR-002 updated in the same commit.

- **Shared types namespace (2026-08-08, orchestrator):** `Entity` renamed
  `DataGraphNode`; `DataGraphEdge` + `DataGraphCombo` added; moved to new
  component `coop-codes.dikwms/type/core-v1` (dir `type/core-v1/`,
  DIKW Data-layer primitives, backend EntityKind parity). Hook + canvas +
  node-drawer consume the types component; hook index re-exports.

- **Namespace singularized (2026-08-08, orchestrator):** `types/` → `type/` —
  component `coop-codes.dikwms/type/core-v1`, dir `dikwms/type/core-v1/`.

- **New component (2026-08-08, orchestrator):** `ui/data-graph/project-page-container`
  (`coop-codes.dikwms.ui.data-graph.project-page-container`) — page shell mirroring
  ProComponents PageContainer API (title/subTitle/extra/breadcrumb/tabs/content/
  footer/ghost/loading), implemented natively on antd 6 (pro-components requires
  antd ^5 — rejected; native keeps the workspace on antd 6). 3 specs green.

- **New context + component (2026-08-08, orchestrator):** `ui/organization/`
  created with `console-page-index-container` (exports
  `OrganizationConsolePageIndexContainer`); check-layout whitelist + docs updated.

- **New context + component (2026-08-08, orchestrator):** `ui/workspace/`
  created with `console-page-index-container` (exports
  `WorkspaceConsolePageIndexContainer`); check-layout whitelist + docs updated.

- **Renamed (2026-08-08, orchestrator):** `ui/data-graph/project-page-container` →
  `ui/data-graph/project-view-container`, export `DataGraphProjectViewContainer`
  (id `coop-codes.dikwms.ui.data-graph.project-view-container`).

- **Project views (2026-08-08, orchestrator):** `ui/data-graph/project-view-container`
  → `project-live-view-container` (export `DataGraphProjectLiveViewContainer`);
  new `ui/project/view-wrapper` exports `ProjectPageWraper` — the project
  view-type seam ('live' now, 'tabular' future via data-tabular context).
  `project` context whitelisted.

- **Renamed (2026-08-08, orchestrator):** `ui/project/view-wrapper` →
  `ui/project/interactive-canvas-wrapper`, export `ProjectLiveViewWraper`;
  `ProjectLiveViewType = 'graph' | 'tabular' | 'geographic'` — graph renders
  `data-graph/project-live-view-container`; tabular/geographic placeholders
  until `data-tabular` / `data-geographic` contexts land.

- **Renamed (2026-08-08, orchestrator):** `ui/data-graph/project-live-view-container` →
  `ui/data-graph/project-interactive-canvas-container` (export
  `DataGraphProjectInteractiveCanvasContainer`); the `ui/project/interactive-canvas-wrapper`
  seam consumes it for `ProjectLiveViewType 'graph'`.

- **New component (2026-08-08, orchestrator):** `ui/project/console-page-index-container`
  (exports `ProjectConsolePageIndexContainer`) — completes the Scope-family
  console index shells (organization/workspace/project).
