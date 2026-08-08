# Feature: SPEC-020-diwkms-frontend-reorg — diwkms Frontend Context Reorganization (Iterative)

<!-- status: Approved -->
<!-- approved-date: 2026-08-08 -->
<!-- approved-by: orchestrator -->
<!-- created: 2026-08-08 -->
<!-- source: backlog SPEC-020 (promoted, narrowed to frontend; orchestrator directive 2026-08-08) -->

> **Living document:** this spec is updated via `## MODIFIED` deltas as each move lands. Status progresses Draft → Approved → Implemented → Archived as the iterative reorg completes. All sections reflect the current truth of the reorg.

## Overview

Iterative process to create and refactor sets of frontend components onto the **`diwkms`** Bit scope and a context-first layout — rename the scope, move the existing components, extract the layout selector, and keep the DDD documentation *reflecting the codebase* (Working software over comprehensive documentation). The number of future components is unknown; this spec defines the process + the initial concrete set.

## Requirements

- REQ-001: Rename Bit scope `coop-codes.network-graph` → `coop-codes.diwkms` (dir `frontend/network-graph/` → `frontend/diwkms/`)
- REQ-002: Move `frontend/network-graph/apps/coop-graph-app` → `frontend/diwkms/app` (bit move, never re-create)
- REQ-003: Move `frontend/network-graph/hooks/use-event-stream` → `frontend/diwkms/hook/use-data-graph-sse` (SSE = Server-Side Events)
- REQ-004: Move `frontend/network-graph/ui/graph` → `frontend/diwkms/ui/data-graph/canvas`
- REQ-005: Move + rename `frontend/network-graph/ui/entity-drawer` → `frontend/diwkms/ui/data-graph/node-drawer`
- REQ-006: New component `frontend/diwkms/ui/data-graph/layout-select` — decouple the layout selector from canvas; keeps `data-testid="layout-select"`
- REQ-007: **Codebase-first docs:** `docs/ddd/bounded-contexts.md` (+ AGENTS.md, ADR-002 as needed) reflects the actual codebase layout, updated in the same commits as the moves — the doc follows the code, never the inverse (Agile Manifesto: working software over comprehensive documentation)
- REQ-008: Iterative process: moves are opportunistic (applied as components are touched); `frontend/scripts/check-layout.mjs` updated to the new shape; spec kept in sync via deltas

## Technical Design

### Target layout (end state)

```
frontend/diwkms/                      (Bit scope: coop-codes.diwkms)
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

- `frontend/workspace.jsonc`: `defaultScope` → `coop-codes.diwkms`; `defaultDirectory` stays `{scope}/{name}`
- Bit: `bit rename --scope` (or per-component `bit move` onto new scope) — component ids become `coop-codes.diwkms/...`
- Dir: `frontend/network-graph/` → `frontend/diwkms/`
- Verify: `bit status` clean; ids resolve to `@coop-codes/diwkms.*`

### Move mechanics (each move)

- `bit move <old-id> <new-path>` — preserves component id dependencies, specs, and test specs; never re-create components
- Name changes ride along where the move IS the rename (e.g. `entity-drawer` → `node-drawer`): rename component function/export + dir in one move
- testid changes ride along ONLY where the component is renamed (node-drawer) or extracted (layout-select); otherwise testids are stable (ADR-003)

### layout-select extraction

- Extract the layout `<Select>` currently at `graph.tsx` (lines ~100–115, `data-testid="layout-select"`, `aria-label="Graph layout type"`) into `ui/data-graph/layout-select`
- Props: `value`, `onChange`, layout options list (9 options today); canvas consumes it
- testid stays `layout-select`; add `layout-select.spec.tsx` covering render + change callback

### check-layout.mjs v2

- Allowed top-level namespaces under `frontend/diwkms/`: `{app, hook, ui}`
- `ui/` entries must be context dirs from whitelist: `data-graph`, `data-graph-governance`, `iam`, `data-schema-registry`
- Each component dir must contain `index.ts(x)` (existing rule kept)
- Landed in the same commit as the first move (never before — must not fail on the current tree)

## Acceptance Criteria

- AC-1: Given the Bit workspace, When the scope is renamed to `coop-codes.diwkms`, Then `bit status` is clean and component ids resolve to `@coop-codes/diwkms.*` (per-component via `bit rename --scope`/`-s`)
- AC-2: Given the app shell, When it is moved to `frontend/diwkms/app`, Then `bit run -p 3100` serves it and the root/`@vite/client`/app modules return 200 (dev port per SPEC-018 R-28)
- AC-3: Given the SSE hook, When moved to `frontend/diwkms/hook/use-data-graph-sse`, Then its spec passes and the smoke canvas still receives live events
- AC-4: Given the canvas, When moved to `frontend/diwkms/ui/data-graph/canvas`, Then the graph renders with the layout selector working and its spec passes
- AC-5: Given the entity drawer, When moved and renamed to `frontend/diwkms/ui/data-graph/node-drawer`, Then the drawer opens from canvas selection, its spec passes, and its testids are renamed `entity-*` → `node-*` (`entity-drawer`→`node-drawer`, `entity-form`→`node-form`, `entity-form-error`→`node-form-error`, `entity-name-input`→`node-name-input`, `entity-save-btn`→`node-save-btn`)
- AC-6: Given the layout selector, When extracted to `frontend/diwkms/ui/data-graph/layout-select` and consumed by canvas, Then `data-testid="layout-select"` remains in the DOM and layout switching still works
- AC-7: Given the reorganized codebase, When docs are checked, Then `docs/ddd/bounded-contexts.md` and `frontend/AGENTS.md` describe the actual layout (scope `diwkms`, `app/`, `hook/`, `ui/<context>/`) — committed in the same commits as the moves
- AC-8: Given the new layout, When `node frontend/scripts/check-layout.mjs` runs, Then it validates `{app, hook, ui}` + context whitelist and reports green
- AC-9: Given each move, When it lands, Then the spec records a `## MODIFIED` delta with the commit hash and AC status — the spec is a living document

## Assumptions & Open Questions

### Assumptions

| # | Assumption | Impact Scope | Verification Method | Status |
|---|-----------|--------------|---------------------|--------|
| A1 | `canva` in the directive was a typo for `canvas` | REQ-004 | confirmed with orchestrator | Verified |
| A2 | New scope id is `coop-codes.diwkms` (org prefix stays) | REQ-001 | confirmed with orchestrator | Verified |
| A3 | Hooks live at `hook/` (singular) with context in the name | REQ-003 | confirmed with orchestrator | Verified |
| A4 | Backend `knowledge-domain` → `data-graph` rename stays out of scope (deferred in backlog) | REQ-008 | confirmed with orchestrator | Verified |
| A5 | testid renames (`entity-*` → `node-*`, five testids verified in code) are cheap now, before SPEC-021 E2E exists | REQ-005 | E2E not implemented | Verified |
| A6 | Component ids follow Bit convention `coop-codes.diwkms/<path>/<name>` | all | bit status clean check | Unverified → AC-1 |

### Open Questions

None — all resolved with the orchestrator (2026-08-08).

## Test Plan

- [ ] `bit test` — full frontend suite green after EVERY move (7 tests today + new layout-select spec)
- [ ] New: `layout-select.spec.tsx` (render + onChange callback)
- [ ] `node frontend/scripts/check-layout.mjs` green (v2, after first move)
- [ ] `bit status` clean; `bit list` shows `coop-codes.diwkms/*` ids
- [ ] Smoke: `bit run` app on dev port — root + modules 200, canvas renders, layout switch works, SSE events flow

## Rollout Plan

Iterative + opportunistic (SPEC-020 policy): each move rides on the next touch of its component — no dedicated batch restructure. Each landed move: code+test+doc delta in one commit, spec delta recorded (AC-9). Order suggested: scope rename first (unblocks id stability), then app → hook → canvas/node-drawer/layout-select as touched.
